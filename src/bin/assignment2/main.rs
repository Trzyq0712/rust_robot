#![no_main]
#![no_std]

use core::cell::RefCell;

use my_hal::{distance::DistanceMeasurer, pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::{asm, interrupt as intr, interrupt::Mutex};
use cortex_m_rt::entry;
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use rtt_target::{rprintln, rtt_init_print};

struct Measurements {
    front: DistanceMeasurer,
    side: DistanceMeasurer,
}

static G_DISTANCES: Mutex<RefCell<Measurements>> = Mutex::new(RefCell::new(Measurements {
    front: DistanceMeasurer::new(),
    side: DistanceMeasurer::new(),
}));

static G_TIM4: Mutex<RefCell<Option<stm32::TIM4>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello");

    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC;
    rcc.ahb1enr.write(|w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.gpiocen().enabled()
    });
    rcc.apb1enr.write(|w| {
        w.tim2en().enabled();
        w.tim4en().enabled()
    });
    rcc.apb2enr.write(|w| w.tim9en().enabled());

    pins::configure_pa2(&dp.GPIOA);
    pins::configure_pa3(&dp.GPIOA);

    pins::configure_pb8(&dp.GPIOB);
    pins::configure_pb9(&dp.GPIOB);

    let tim2 = dp.TIM2;
    timers::configure_tim2(&tim2);
    tim2.cr1.modify(|_, w| w.cen().enabled());

    let tim4 = dp.TIM4;
    timers::configure_tim4(&tim4);
    tim4.cr1.modify(|_, w| w.cen().enabled());

    let tim9 = dp.TIM9;
    timers::configure_tim9(&tim9);

    dp.GPIOC.moder.modify(|_, w| w.moder13().output());
    dp.GPIOC.otyper.modify(|_, w| w.ot13().push_pull());
    dp.GPIOC.odr.modify(|_, w| w.odr13().high());

    intr::free(|cs| {
        G_TIM4.borrow(cs).replace(Some(tim4));
    });

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM4);
    }

    loop {
        dp.GPIOC.odr.modify(|r, w| w.odr13().bit(!r.odr13().bit()));
        let front_dist = intr::free(|cs| G_DISTANCES.borrow(cs).borrow().front.get_distance_cm());
        rprintln!("Distance: {}", front_dist);
        asm::delay(2_000_000);
    }
}

#[interrupt]
fn TIM4() {
    let (front, side) = intr::free(|cs| {
        let tim4 = G_TIM4.borrow(cs).take().unwrap();
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
        G_TIM4.borrow(cs).replace(Some(tim4));
        (front, side)
    });
    intr::free(|cs| {
        let mut distances = G_DISTANCES.borrow(cs).borrow_mut();
        front
            .into_iter()
            .for_each(|t| distances.front.update_measurment(t));
        side.into_iter()
            .for_each(|t| distances.side.update_measurment(t));
    });
}
