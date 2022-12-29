#![no_main]
#![no_std]
#![allow(unused)]

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::{asm, interrupt as intr, interrupt::Mutex};
use cortex_m_rt::{entry, exception};
use rtt_target::{rprintln, rtt_init_print};
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use my_hal::{pins, timers};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello2");

    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    dp.RCC
        .ahb1enr
        .modify(|_, w| w.gpiocen().enabled().gpioben().enabled());
    dp.RCC.apb1enr.modify(|_, w| w.tim2en().enabled());

    dp.GPIOC.moder.modify(|_, w| w.moder13().output());
    dp.GPIOC.otyper.modify(|_, w| w.ot13().push_pull());

    pins::configure_led_indicators(&dp.GPIOB);
    timers::configure_tim2(&dp.TIM2);

    loop {
        rprintln!("Loop");
        timers::set_duty_tim2(&dp.TIM2, 3 - 1, 20);
        rprintln!("duty: {}", dp.TIM2.ccr3().read().bits());
        asm::delay(20_000_000);
        timers::set_duty_tim2(&dp.TIM2, 3 - 1, 0);
        rprintln!("duty: {}", dp.TIM2.ccr3().read().bits());
        asm::delay(20_000_000);
        timers::set_duty_tim2(&dp.TIM2, 3 - 1, 70);
        rprintln!("duty: {}", dp.TIM2.ccr3().read().bits());
        asm::delay(20_000_000);
    }
}
