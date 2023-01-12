use stm32f4::stm32f401::{TIM1, TIM10, TIM2, TIM3, TIM4, TIM5, TIM9};

const AUTO_RELOAD: u16 = 1 << 11;

// the internal clock is running at 16MHz

/// configure tim1 to serve as a trigger for the proximity sensor
pub fn configure_tim1(tim: &TIM1) {
    // we need to generate a pulse of 10us (1e-5s)
    // 16e6 / 1e5 = 16
    // the arr should be at least 16, let us set it to 32 to be on the safe side
    tim.cr1
        .write(|w| w.opm().enabled().arpe().enabled().urs().any_event()); // enable one-pulse mode
    tim.cr2.write(|w| w.ois3n().clear_bit().ois3().set_bit());
    tim.smcr.write(|w| {
        w.sms().trigger_mode(); // trigger the counting on the rising edge
        w.ts().itr0() // use tim5 as the trigger
    });
    tim.ccmr1_input().write(|w| w.cc1s().trc()); // use the internal event as the trigger

    tim.psc.write(|w| w.psc().bits(1000));
    tim.arr.write(|w| w.arr().bits(1 << 9)); // set the length of the pulse
    tim.ccr3().write(|w| w.ccr().bits(1 << 9)); // we do not want a delay before the pulse
    tim.ccmr2_output()
        .write(|w| w.oc3m().pwm_mode2().cc3s().output().oc3pe().enabled());
    tim.ccer.write(|w| w.cc3e().set_bit());
    tim.bdtr.write(|w| w.moe().enabled());
    tim.dier.write(|w| w.cc3ie().enabled().uie().enabled());
    tim.egr.write(|w| w.ug().update());
}

/// configure tim2 to serve as a pwm for modulated output for peripherals
pub fn configure_tim2_(tim: &TIM2) {
    tim.arr.modify(|_, w| w.arr().bits(AUTO_RELOAD as u32));
    tim.ccr.iter().for_each(|ccr| ccr.modify(|_, w| w.bits(0)));
    tim.ccer.modify(|_, w| w.cc3e().set_bit());
    tim.ccmr2_output().modify(|_, w| {
        w.oc3m().pwm_mode1();
        w.cc3s().output()
    })
}

/// configure tim4 to measure the pulse lenghts of the proximity sensors
pub fn configure_tim4_(tim: &TIM4) {
    tim.psc.write(|w| w.psc().bits(u16::max_value())); // set to 7
    tim.cr1.write(|w| w.urs().any_event().arpe().enabled());
    tim.cr2.write(|w| w.ti1s().xor()); // use CH3 as TI1
    tim.ccmr1_input().write(|w| {
        w.cc1s().ti1(); // TI1 to supply the CC1
        w.ic1f().bits(1) // not doing any sample filtering
    });
    tim.smcr.write(|w| {
        w.ts().ti1fp1();
        w.sms().reset_mode()
    });
    tim.ccmr2_input().write(|w| w.cc3s().ti3()); // select TI3 for the CC3
    tim.ccer.write(|w| {
        // detect a rising edge transition to start counting
        w.cc1p().clear_bit();
        w.cc1np().clear_bit();
        // detect a falling edge transition to stop counting
        w.cc3p().set_bit();
        w.cc3np().clear_bit()
    });
    tim.dier.write(|w| w.cc3ie().enabled()); // enable interrupts on capture
    tim.egr.write(|w| w.ug().update());
}

/// configure tim5 to trigger the sending of triggers to the proximity sensors (by tim1)
pub fn configure_tim5(tim: &TIM5) {
    tim.arr.write(|w| w.arr().bits(16_000_000 / 16)); // should trigger tim1 every 2 seconds
    tim.cr2.write(|w| w.mms().update()); // send the trigger on update event
}
/// Sets duty of the channel.
/// duty is between 0 and 100
pub fn set_duty_tim2(tim: &TIM2, channel: u8, duty: u8) {
    let max_duty = tim.arr.read().arr().bits();
    tim.ccr[channel as usize].write(|w| w.bits(max_duty * duty as u32 / 100));
}

pub fn configure_tim3(tim: &TIM3) {
    tim.ccmr1_output().write(|w| {
        w.cc1s().output();
        w.cc2s().output();
        w.oc1m().pwm_mode1();
        w.oc2m().pwm_mode1()
    });
    tim.ccmr2_output().write(|w| {
        w.cc3s().output();
        w.cc4s().output();
        w.oc3m().pwm_mode1();
        w.oc4m().pwm_mode1()
    });
    tim.ccer.write(|w| {
        w.cc1e().set_bit();
        w.cc2e().set_bit();
        w.cc3e().set_bit();
        w.cc4e().set_bit()
    });
    tim.arr.write(|w| w.arr().bits(u16::MAX));
}

pub enum Direction {
    Forward,
    Backward,
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

/// Configure TIM9 to send a pulse for the ultrasonic sensor.
pub fn configure_tim9(tim: &TIM9) {
    tim.cr1.write(|w| w.opm().enabled()); // enable one-pulse mode
    tim.smcr.write(|w| unsafe {
        w.ts().bits(0b000); // use TIM2 as the trigger
        w.sms().bits(0b110) // enable slave trigger mode
    });

    // We want a pulse of at least 10us.
    // At 16MHz we need a pulse that lasts at least 160 ticks.
    tim.arr.write(|w| unsafe { w.arr().bits(1 << 8) });
    tim.ccr
        .iter()
        .for_each(|ccr| ccr.write(|w| unsafe { w.ccr().bits(1) }));

    tim.dier.write(|w| w.uie().enabled());
    // Enable PWM Mode 1
    tim.ccmr1_output().write(|w| unsafe {
        w.oc1m().bits(0b111);
        w.cc1s().bits(0b00);
        w.oc2m().bits(0b111);
        w.cc2s().bits(0b00)
    });
    // Enable the output channels.
    tim.ccer.write(|w| w.cc1e().set_bit().cc2e().set_bit());
}

/// Configure TIM2 to trigger TIM9 pulse.
pub fn configure_tim2(tim: &TIM2) {
    tim.cr2.write(|w| w.mms().update());
    tim.arr.write(|w| w.arr().bits(1 << 24));
}

/// Configure TIM4 to measure pulse lengths of the ultrasound sensors.
pub fn configure_tim4(tim: &TIM4) {
    // Enable cc interrupts
    tim.dier.write(|w| w.cc3ie().enabled().cc4ie().enabled());
    // Select input channels for cc triggers
    tim.ccmr2_input().write(|w| {
        w.cc3s()
            .ti3()
            .cc4s()
            .ti4()
            .ic3f()
            .bits(0b0110)
            .ic4f()
            .bits(0b0110)
    });
    tim.arr.write(|w| w.arr().bits(u16::MAX));
    tim.ccer.write(|w| {
        // enable channels
        w.cc3e().set_bit();
        w.cc4e().set_bit();
        // trigger at both rising and falling edge
        w.cc3p().set_bit();
        w.cc3np().set_bit();
        w.cc4p().set_bit();
        w.cc4np().set_bit()
    });
}
