#![no_main]
#![no_std]

use my_hal::robot::{Robot, SensorReadings};
use my_hal::states::State;
use my_hal::{adc, dma, pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m_rt::entry;
use stm32f4::stm32f401 as stm32;

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC;
    rcc.ahb1enr.write(|w| {
        w.gpioaen().enabled();
        w.gpioben().enabled();
        w.dma2en().enabled()
    });
    rcc.apb1enr.write(|w| w.tim3en().enabled());
    rcc.apb2enr.write(|w| w.adc1en().enabled());

    pins::configure_motor_pins(&dp.GPIOB);

    let tim3 = dp.TIM3;
    timers::configure_tim3(&tim3);
    tim3.cr1.modify(|_, w| w.cen().enabled());

    pins::configure_pa4(&dp.GPIOA);
    pins::configure_pa5(&dp.GPIOA);

    let dma2 = dp.DMA2;
    dma::configure_dma2(&dma2);
    dma2.st[0].cr.modify(|_, w| w.en().enabled());

    let adc1 = dp.ADC1;
    adc::configure_adc(&adc1);
    adc1.cr2.modify(|_, w| w.swstart().start());

    let mut robot = Robot::default();
    let mut state = State::FollowingLine;

    loop {
        let readings = unsafe { adc::INFRARED };
        let readings = SensorReadings {
            left_infrared: readings[0],
            right_infrared: readings[1],
            ..Default::default()
        };
        robot.update_sensors(readings);
        state = state.process_state(&mut robot);
    }
}
