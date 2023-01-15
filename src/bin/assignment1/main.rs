#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::asm;
use cortex_m_rt::entry;
use stm32f4::stm32f401 as stm32;

use my_hal::{pins, timers};

fn delay_ms(ms: u32) {
    asm::delay(16_000 * ms);
}

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    dp.RCC.ahb1enr.modify(|_, w| w.gpioben().enabled());
    dp.RCC.apb1enr.modify(|_, w| w.tim3en().enabled());

    pins::configure_motor_pins(&dp.GPIOB);
    let tim3 = dp.TIM3;
    timers::configure_tim3(&tim3);
    tim3.cr1.modify(|_, w| w.cen().enabled());

    loop {
        use timers::Direction::*;
        timers::set_left_motor_duty(&tim3, u16::MAX / 2, Forward);
        timers::set_right_motor_duty(&tim3, u16::MAX / 2, Forward);
        delay_ms(3000);
        timers::set_left_motor_duty(&tim3, u16::MAX, Forward);
        timers::set_right_motor_duty(&tim3, u16::MAX, Forward);
        delay_ms(4000);
        for duty in (0..=u16::MAX).step_by(1 << 11).rev() {
            timers::set_left_motor_duty(&tim3, duty, Forward);
            timers::set_right_motor_duty(&tim3, duty, Forward);
            delay_ms(200);
        }
    }
}
