#![no_main]
#![no_std]

use core::cell::RefCell;

use cortex_m::asm;
use my_hal::robot::{Motor, Robot};
use my_hal::{adc, dma, pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::{interrupt as intr, interrupt::Mutex};
use cortex_m_rt::entry;
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use rtt_target::{rprintln, rtt_init_print};

static G_TIM1: Mutex<RefCell<Option<stm32::TIM1>>> = Mutex::new(RefCell::new(None));

static G_ROBOT: Mutex<RefCell<Option<Robot>>> = Mutex::new(RefCell::new(None));

static mut INFRARED: [u16; 2] = [0, 0];

const DISTANCE: u16 = 120;

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
        w.tim3en().enabled();
        w.tim2en().enabled();
        w.tim4en().enabled()
    });
    rcc.apb2enr.write(|w| {
        w.tim1en().enabled();
        w.adc1en().enabled()
    });

    pins::configure_motor_pins(&dp.GPIOB);

    pins::configure_pa0(&dp.GPIOA);
    pins::configure_pa1(&dp.GPIOA);

    pins::configure_pa8(&dp.GPIOA);

    let tim1 = dp.TIM1;
    timers::configure_tim1(&tim1);

    let tim2 = dp.TIM2;
    timers::configure_tim2(&tim2);
    tim2.cr1.modify(|_, w| w.cen().enabled());

    let tim3 = dp.TIM3;
    timers::configure_tim3(&tim3);
    tim3.cr1.modify(|_, w| w.cen().enabled());

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
    robot.left_motor().forward(u16::MAX);
    robot.right_motor().forward(u16::MAX - 11_500);
    tim1.arr.write(|w| w.arr().bits(DISTANCE * 20 / 21));

    intr::free(|cs| {
        G_TIM1.borrow(cs).replace(Some(tim1));
        G_ROBOT.borrow(cs).replace(Some(robot));
    });

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM1_UP_TIM10);
    }

    loop {
        // let (cnt, arr) = intr::free(|cs| {
        //     let some_tim1 = G_TIM1.borrow(cs).borrow();
        //     let tim1 = some_tim1.as_ref().unwrap();
        //     (tim1.cnt.read().cnt().bits(), tim1.arr.read().arr().bits())
        // });
        asm::nop();
    }
}

#[interrupt]
fn TIM1_UP_TIM10() {
    intr::free(|cs| {
        let mut some_tim1 = G_TIM1.borrow(cs).borrow_mut();
        let tim1 = some_tim1.as_mut().unwrap();
        tim1.sr.modify(|_, w| w.uif().clear());
    });

    intr::free(|cs| {
        let mut some_robot = G_ROBOT.borrow(cs).borrow_mut();
        let robot = some_robot.as_mut().unwrap();
        robot.left_motor().stop();
        robot.right_motor().stop();
    })
}
