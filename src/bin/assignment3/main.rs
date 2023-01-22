#![no_main]
#![no_std]

use cortex_m::asm;
use my_hal::robot::{Motor, Robot, SensorReadings};
use my_hal::states::State;
use my_hal::{adc, dma, pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

// use cortex_m::{interrupt as intr, interrupt::Mutex};
use cortex_m_rt::entry;
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

    let left_motor = unsafe {
        Motor::new(
            tim3.arr.as_ptr() as *const u16,
            tim3.ccr3().as_ptr() as *mut u16,
            tim3.ccr4().as_ptr() as *mut u16,
        )
    };

    let right_motor = unsafe {
        Motor::new(
            tim3.arr.as_ptr() as *const u16,
            tim3.ccr1().as_ptr() as *mut u16,
            tim3.ccr2().as_ptr() as *mut u16,
        )
    };

    let mut robot = Robot::new(left_motor, right_motor);

    let mut state = State::FollowingLine;

    loop {
        let readings = unsafe { INFRARED };
        let readings = SensorReadings {
            left_infrared: readings[0],
            right_infrared: readings[1],
            ..Default::default()
        };
        robot.update_sensors(readings);
        state = state.process_state(&mut robot);
    }
}
