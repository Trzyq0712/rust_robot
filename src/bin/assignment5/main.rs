#![no_main]
#![no_std]

use cortex_m::asm;
use my_hal::adc;
use my_hal::dma;
use my_hal::robot::Robot;
use my_hal::robot::SensorReadings;
use my_hal::states::State;
use my_hal::timers::{G_TIM2, G_TIM5};
use my_hal::{pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::interrupt::free;
use cortex_m_rt::entry;
use stm32f4::stm32f401 as stm32;

const DISTANCE: u32 = 27 * 5;

#[entry]
fn main() -> ! {
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
        w.tim5en().enabled()
    });
    rcc.apb2enr.write(|w| w.adc1en().enabled());

    pins::configure_motor_pins(&dp.GPIOB);
    pins::configure_pa0(&dp.GPIOA);
    pins::configure_pa1(&dp.GPIOA);
    pins::configure_pa4(&dp.GPIOA);
    pins::configure_pa5(&dp.GPIOA);

    timers::configure_tim3(&dp.TIM3);
    timers::configure_tim2(&dp.TIM2);
    timers::configure_tim5(&dp.TIM5);

    timers::init_global_timers(dp.TIM4, dp.TIM2, dp.TIM5);

    let dma2 = dp.DMA2;
    dma::configure_dma2(&dma2);
    dma2.st[0].cr.modify(|_, w| w.en().enabled());

    let adc1 = dp.ADC1;
    adc::configure_adc(&adc1);
    adc1.cr2.modify(|_, w| w.swstart().start());

    let distance = DISTANCE * 200 / 213;
    // Correction for some back turns
    let distance = distance * 109 / 100;

    let mut robot = Robot::default();
    let mut state = State::FollowingLine;
    robot.lock_left_motor(distance);
    robot.lock_right_motor(distance);

    while !should_stop() {
        let [left, right] = unsafe { adc::INFRARED };
        let new_readings = SensorReadings {
            left_infrared: left,
            right_infrared: right,
            ..Default::default()
        };
        robot.update_sensors(new_readings);
        state = state.process_state(&mut robot);
    }
    robot.left_motor().stop();
    robot.right_motor().stop();

    loop {
        asm::nop();
    }
}

fn should_stop() -> bool {
    free(|cs| {
        G_TIM2
            .borrow(cs)
            .borrow()
            .as_ref()
            .unwrap()
            .cr1
            .read()
            .cen()
            .is_disabled()
    }) && free(|cs| {
        G_TIM5
            .borrow(cs)
            .borrow()
            .as_ref()
            .unwrap()
            .cr1
            .read()
            .cen()
            .is_disabled()
    })
}
