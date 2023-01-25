#![no_main]
#![no_std]
use cortex_m::asm;
use cortex_m::interrupt::{free, Mutex};
use my_hal::robot::{Robot, SensorReadings};
use my_hal::states::State;
use my_hal::{
    adc,
    distance::{self, DistanceMeasurer},
    dma, pins, timers,
};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use rtt_target::{rprintln, rtt_init_print};

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
        w.tim4en().enabled();
        w.tim5en().enabled()
    });
    rcc.apb2enr.write(|w| {
        w.adc1en().enabled();
        w.tim9en().enabled()
    });
    pins::configure_motor_pins(&dp.GPIOB);
    pins::configure_ultrasound_pins(&dp.GPIOA, &dp.GPIOB);
    pins::configure_pa0(&dp.GPIOA);
    pins::configure_pa1(&dp.GPIOA);
    pins::configure_pa4(&dp.GPIOA);
    pins::configure_pa5(&dp.GPIOA);

    timers::configure_tim3(&dp.TIM3);
    timers::configure_tim4(&dp.TIM4);
    timers::configure_tim2(&dp.TIM2);
    timers::configure_tim5(&dp.TIM5);
    timers::configure_tim9(&dp.TIM9);

    dma::configure_dma2(&dp.DMA2);
    dp.DMA2.st[0].cr.modify(|_, w| w.en().enabled());

    adc::configure_adc(&dp.ADC1);
    dp.ADC1.cr2.modify(|_, w| w.swstart().start());

    timers::init_global_timers(dp.TIM4, dp.TIM2, dp.TIM5);
    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM4);
        stm32::NVIC::unmask(stm32::interrupt::TIM2);
        stm32::NVIC::unmask(stm32::interrupt::TIM5);
    }

    let mut robot = Robot::default();
    let mut state = State::FollowingLineAndAvoiding;

    loop {
        let readings = unsafe { adc::INFRARED };
        let (front_dist, left_dist) = free(|cs| {
            let dis = distance::G_DISTANCES.borrow(cs).borrow();
            (dis.front.get_distance_cm(), dis.side.get_distance_cm())
        });
        let readings = SensorReadings {
            left_infrared: readings[0],
            right_infrared: readings[1],
            front_distance: front_dist,
            left_distance: left_dist,
        };
        // rprintln!("{:?}", &state);
        // rprintln!("{:?}", &readings);
        robot.update_sensors(readings);
        state = state.process_state(&mut robot);
        // asm::delay(1_000_000);
    }
}

#[interrupt]
fn TIM4() {
    timers::tim4_interrupt_handler();
}

#[interrupt]
fn TIM2() {
    timers::tim2_interrupt_handler();
}

#[interrupt]
fn TIM5() {
    timers::tim5_interrupt_handler();
}
