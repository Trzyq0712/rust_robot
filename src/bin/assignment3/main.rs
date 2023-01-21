#![no_main]
#![no_std]

use core::mem;
use cortex_m::asm;
use my_hal::{adc, dma, pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

// use cortex_m::{interrupt as intr, interrupt::Mutex};
use cortex_m_rt::entry;
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use rtt_target::{rprintln, rtt_init_print};

static mut INFRARED: [u16; 2] = [0, 0];

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC;
    rcc.ahb1enr.write(|w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.dma2en().enabled()
    });
    rcc.apb1enr.write(|w| {
        w.tim2en().enabled();
        w.tim3en().enabled();
        w.tim4en().enabled()
    });
    rcc.apb2enr.write(|w| w.adc1en().enabled());

    pins::configure_motor_pins(&dp.GPIOB);

    let tim2 = dp.TIM2;
    timers::configure_tim2(&tim2);
    tim2.cr1.modify(|_, w| w.cen().enabled());

    let tim3 = dp.TIM3;
    timers::configure_tim3(&tim3);
    tim3.cr1.modify(|_, w| w.cen().enabled());

    pins::configure_pa0(&dp.GPIOA);
    pins::configure_pa1(&dp.GPIOA);

    let dma2 = dp.DMA2;
    dma::configure_dma2(&dma2, unsafe { INFRARED.as_ptr() });
    dma2.st[0].cr.modify(|_, w| w.en().enabled());

    let adc1 = dp.ADC1;
    adc::configure_adc(&adc1);
    adc1.cr2.modify(|_, w| w.swstart().start());

    const LEFT_MOTOR_DUTY: u16 = (u16::MAX as f32 * 0.9) as u16;
    // adjust max duty to go straight
    const RIGHT_MOTOR_DUTY: u16 = (u16::MAX as f32 * 0.75 * 0.9) as u16;

    const MEASUREMENTS: usize = 16;
    let mut center_pos = 0;
    for _ in 0..MEASUREMENTS {
        let avg = unsafe { INFRARED }.iter().sum::<u16>();
        center_pos += avg as u32;
        asm::delay(1000);
    }
    let center_pos: u16 = (center_pos / MEASUREMENTS as u32 / 2) as u16;

    const KP: f32 = 200.0;
    const KI: f32 = 5.0;
    const FD: f32 = 50.0;

    let mut left_duty = LEFT_MOTOR_DUTY as i32;
    let mut right_duty = RIGHT_MOTOR_DUTY as i32;

    let mut sum_of_errors = 0;
    let mut prev_error = 0;

    rprintln!("{}", center_pos);

    loop {
        let readings = unsafe { INFRARED };
        let error = center_pos - readings[0];
        let adjustment =
            KP * error as f32 + KI * sum_of_errors as f32 + FD * (prev_error - error) as f32;
        left_duty += adjustment as i32;
        right_duty -= adjustment as i32;
        timers::set_left_motor(&tim3, left_duty);
        timers::set_right_motor(&tim3, right_duty);

        prev_error = error;
        sum_of_errors += error;

        asm::delay(200);
    }
}
