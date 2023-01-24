#![no_main]
#![no_std]

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::interrupt::free;
use cortex_m_rt::entry;
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use my_hal::{distance::G_DISTANCES, pins, timers};

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC;
    rcc.ahb1enr.write(|w| {
        w.gpioaen().enabled();
        w.gpioben().enabled()
    });
    rcc.apb1enr.write(|w| {
        w.tim3en().enabled();
        w.tim4en().enabled()
    });
    rcc.apb2enr.write(|w| w.tim9en().enabled());

    pins::configure_motor_pins(&dp.GPIOB);

    pins::configure_ultrasound_pins(&dp.GPIOA, &dp.GPIOB);

    timers::configure_tim3(&dp.TIM3);
    timers::configure_tim4(&dp.TIM4);
    timers::configure_tim9(&dp.TIM9);

    timers::init_global_timers(dp.TIM4, dp.TIM2, dp.TIM5);

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM4);
    }

    const LEFT_MOTOR_DUTY: u16 = u16::MAX;
    // adjust max duty to go straight
    const RIGHT_MOTOR_DUTY: u16 = (u16::MAX as f32 * 0.82) as u16;

    loop {
        let front_dist = free(|cs| G_DISTANCES.borrow(cs).borrow().front.get_distance_cm());
        let duties = if front_dist < 15 {
            (0, 0)
        } else {
            (LEFT_MOTOR_DUTY, RIGHT_MOTOR_DUTY)
        };
        timers::set_left_motor_duty(&dp.TIM3, duties.0, timers::Direction::Forward);
        timers::set_right_motor_duty(&dp.TIM3, duties.1, timers::Direction::Forward);
    }
}

#[interrupt]
fn TIM4() {
    timers::tim4_interrupt_handler();
}
