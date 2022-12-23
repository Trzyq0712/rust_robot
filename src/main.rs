#![allow(unused)]
#![no_main]
#![no_std]

// Halt on panic
use core::f32::consts::FRAC_PI_2;
use cortex_m::asm::delay;
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use micromath::F32Ext;
use stm32f4xx_hal as hal;

use crate::hal::timer::Channel;
use crate::hal::{pac, prelude::*};
use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello2");

    let dp = pac::Peripherals::take().unwrap();
    let gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output();

    let gpiob = dp.GPIOB.split();
    let gpioa = dp.GPIOA.split();
    let mut left_motor_fd = gpiob.pb15.into_push_pull_output();
    let mut left_motor_bk = gpiob.pb14.into_push_pull_output();
    let mut right_motor_bk = gpiob.pb3.into_push_pull_output();
    let mut right_motor_fd = gpioa.pa8.into_push_pull_output();

    // Set up the system clock. We want to run at 48MHz for this one.
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.use_hse(25.MHz()).sysclk(48.MHz()).freeze();

    // Create a delay abstraction based on general-pupose 32-bit timer TIM5
    let mut delay = dp.TIM5.delay_us(&clocks);

    // let channels = (
    //     gpiob.pb0.into_alternate::<2>(),
    //     gpiob.pb1.into_alternate::<2>(),
    // );

    // let mut pwm = dp.TIM3.pwm_us(channels, 0xFF.micros(), &clocks);
    // let max_duty = pwm.get_max_duty();

    // rprintln!("Max duty: {}", &max_duty);

    // pwm.enable(Channel::C3);
    // pwm.enable(Channel::C4);

    left_motor_fd.set_high();
    left_motor_bk.set_low();
    right_motor_fd.set_high();
    right_motor_bk.set_low();

    loop {
        cortex_m::asm::nop();
        delay.delay(1.secs());
        led.toggle();
    }
}
