use core::cell::RefCell;
use cortex_m::interrupt::{free, Mutex};
use stm32f4::stm32f401::{TIM2, TIM3, TIM4, TIM5, TIM9};

// The internal clock is running at 16MHz.

pub static G_TIM4: Mutex<RefCell<Option<TIM4>>> = Mutex::new(RefCell::new(None));
pub static G_TIM2: Mutex<RefCell<Option<TIM2>>> = Mutex::new(RefCell::new(None));
pub static G_TIM5: Mutex<RefCell<Option<TIM5>>> = Mutex::new(RefCell::new(None));

/// Configure TIM3 to manage PWM for the motors.
pub fn configure_tim3(tim: &TIM3) {
    // Configure channels as outputs in PWM mode
    tim.ccmr1_output().write(|w| {
        w.cc1s().output();
        w.cc2s().output();
        w.oc1m().pwm_mode1();
        w.oc2m().pwm_mode1()
    });
    // Configure channels as outputs in PWM mode
    tim.ccmr2_output().write(|w| {
        w.cc3s().output();
        w.cc4s().output();
        w.oc3m().pwm_mode1();
        w.oc4m().pwm_mode1()
    });
    // Enable all channels.
    tim.ccer.write(|w| {
        w.cc1e().set_bit();
        w.cc2e().set_bit();
        w.cc3e().set_bit();
        w.cc4e().set_bit()
    });
    tim.arr.write(|w| w.arr().bits(u16::MAX));
    tim.cr1.modify(|_, w| w.cen().enabled());
}

/// Configure TIM9 to send a pulse every 70ms to the ultrasonic sensor.
pub fn configure_tim9(tim: &TIM9) {
    tim.psc.write(|w| w.psc().bits(159)); // Each tick is 10us
    tim.arr.write(|w| unsafe { w.arr().bits(7000) }); // Period of 70ms
    tim.ccr
        .iter()
        .for_each(|ccr| ccr.write(|w| unsafe { w.ccr().bits(2) })); // We just need > 10us

    tim.ccmr1_output().write(|w| unsafe {
        // Configure channels as outputs
        w.cc1s().bits(0b00);
        w.cc2s().bits(0b00);
        // Enable PWM Mode 1
        w.oc1m().bits(0b111);
        w.oc2m().bits(0b111)
    });
    // Enable the output channels.
    tim.ccer.write(|w| w.cc1e().set_bit().cc2e().set_bit());
    tim.cr1.modify(|_, w| w.cen().enabled());
}

/// Configure TIM4 to measure pulse lengths of the ultrasound sensors.
pub fn configure_tim4(tim: &TIM4) {
    // Enable cc interrupts
    tim.dier.write(|w| w.cc3ie().enabled().cc4ie().enabled());
    // Select input channels for CC triggers
    tim.ccmr2_input().write(|w| {
        w.cc3s()
            .ti3()
            .cc4s()
            .ti4()
            .ic3f()
            .bits(0b0111)
            .ic4f()
            .bits(0b0111)
    });
    tim.arr.write(|w| w.arr().bits(u16::MAX));
    tim.psc.write(|w| w.psc().bits(15));
    tim.ccer.write(|w| {
        // Enable channels
        w.cc3e().set_bit();
        w.cc4e().set_bit();
        // Trigger at both rising and falling edge
        w.cc3p().set_bit();
        w.cc3np().set_bit();
        w.cc4p().set_bit();
        w.cc4np().set_bit()
    });
    tim.cr1.modify(|_, w| w.cen().enabled());
}

/// Configure TIM2 to make the left motor run a certain number of rotations.
pub fn configure_tim2(tim: &TIM2) {
    tim.cr1.write(|w| w.opm().enabled());
    tim.smcr.write(|w| {
        w.ts().ti1fp1();
        w.sms().ext_clock_mode()
    });
    tim.ccmr1_input().write(|w| {
        w.cc1s().ti1();
        w.ic1f().bits(0b1111)
    });
    tim.ccer.write(|w| {
        w.cc1p().clear_bit();
        w.cc1np().clear_bit()
    });
    tim.dier.write(|w| w.uie().enabled());
}

/// Configure TIM5 to make the right motor run a certain number of rotations.
pub fn configure_tim5(tim: &TIM5) {
    tim.cr1.write(|w| w.opm().enabled());
    tim.smcr.write(|w| {
        w.ts().ti2fp2();
        w.sms().ext_clock_mode()
    });
    tim.ccmr1_input().write(|w| {
        w.cc2s().ti2();
        w.ic2f().bits(0b1111)
    });
    tim.ccer.write(|w| {
        w.cc1p().clear_bit();
        w.cc1np().clear_bit()
    });
    tim.dier.write(|w| w.uie().enabled());
}

pub fn tim4_interrupt_handler() {
    let (front, side) = free(|cs| {
        let some_tim4 = G_TIM4.borrow(cs).borrow();
        let tim4 = some_tim4.as_ref().unwrap();
        let sr = &tim4.sr;
        let front = sr
            .read()
            .cc3if()
            .bit_is_set()
            .then_some(tim4.ccr3().read().ccr().bits());
        let side = sr
            .read()
            .cc4if()
            .bit_is_set()
            .then_some(tim4.ccr4().read().ccr().bits());
        sr.modify(|_, w| {
            w.cc3of()
                .clear_bit()
                .cc4of()
                .clear_bit()
                .cc3if()
                .clear_bit()
                .cc4if()
                .clear_bit()
        });
        (front, side)
    });
    free(|cs| {
        use crate::distance::G_DISTANCES;
        let mut distances = G_DISTANCES.borrow(cs).borrow_mut();
        front
            .into_iter()
            .for_each(|t| distances.front.update_measurment(t));
        side.into_iter()
            .for_each(|t| distances.side.update_measurment(t));
    });
}

pub fn tim2_interrupt_handler() {
    free(|cs| {
        G_TIM2
            .borrow(cs)
            .borrow()
            .as_ref()
            .unwrap()
            .sr
            .modify(|_, w| w.uif().clear());
        let tim3 = unsafe { &*TIM3::PTR };
        tim3.ccr3().reset();
        tim3.ccr4().reset();
    });
}

pub fn tim5_interrupt_handler() {
    free(|cs| {
        G_TIM5
            .borrow(cs)
            .borrow()
            .as_ref()
            .unwrap()
            .sr
            .modify(|_, w| w.uif().clear());
        let tim3 = unsafe { &*TIM3::PTR };
        tim3.ccr1().reset();
        tim3.ccr2().reset();
    });
}

pub fn init_global_timers(tim4: TIM4, tim2: TIM2, tim5: TIM5) {
    free(|cs| {
        G_TIM4.borrow(cs).replace(Some(tim4));
        G_TIM2.borrow(cs).replace(Some(tim2));
        G_TIM5.borrow(cs).replace(Some(tim5));
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

pub fn set_left_motor(tim: &TIM3, mut duty: i32) {
    duty = duty.clamp(-(u16::MAX as i32), u16::MAX as i32);
    let duties = if duty > 0 { (duty, 0) } else { (0, duty) };
    tim.ccr3().write(|w| w.ccr().bits(duties.0 as u16));
    tim.ccr4().write(|w| w.ccr().bits(duties.1 as u16));
}

pub fn set_right_motor(tim: &TIM3, mut duty: i32) {
    duty = duty.clamp(-(u16::MAX as i32), u16::MAX as i32);
    let duties = if duty > 0 { (duty, 0) } else { (0, duty) };
    tim.ccr1().write(|w| w.ccr().bits(duties.0 as u16));
    tim.ccr2().write(|w| w.ccr().bits(duties.1 as u16));
}

pub fn set_left_motor_duty(tim: &TIM3, duty: u16, direction: Direction) {
    let values = match direction {
        Direction::Forward => (duty, 0),
        Direction::Backward => (0, duty),
    };
    tim.ccr3().write(|w| w.ccr().bits(values.0));
    tim.ccr4().write(|w| w.ccr().bits(values.1));
}

pub fn set_right_motor_duty(tim: &TIM3, duty: u16, direction: Direction) {
    let values = match direction {
        Direction::Forward => (duty, 0),
        Direction::Backward => (0, duty),
    };
    tim.ccr1().write(|w| w.ccr().bits(values.0));
    tim.ccr2().write(|w| w.ccr().bits(values.1));
}
