#![no_main]
#![no_std]

use cortex_m::asm;
use cortex_m::{interrupt as intr, interrupt::Mutex};
use my_hal::robot::{Motor, Robot, SensorReadings};
use my_hal::states::State;
use my_hal::{adc, distance::DistanceMeasurer, dma, pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

// use cortex_m::{interrupt as intr, interrupt::Mutex};
use core::cell::RefCell;
use cortex_m_rt::entry;
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use rtt_target::{rprintln, rtt_init_print};

struct Measurements {
    front: DistanceMeasurer,
    side: DistanceMeasurer,
}

static G_DISTANCES: Mutex<RefCell<Measurements>> = Mutex::new(RefCell::new(Measurements {
    front: DistanceMeasurer::new(),
    side: DistanceMeasurer::new(),
}));

static G_TIM4: Mutex<RefCell<Option<stm32::TIM4>>> = Mutex::new(RefCell::new(None));

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
    rcc.apb2enr.write(|w| {
        w.adc1en().enabled();
        w.tim9en().enabled()
    });

    pins::configure_motor_pins(&dp.GPIOB);

    pins::configure_pa2(&dp.GPIOA);
    pins::configure_pa3(&dp.GPIOA);

    pins::configure_pb8(&dp.GPIOB);
    pins::configure_pb9(&dp.GPIOB);

    let tim2 = dp.TIM2;
    timers::configure_tim2(&tim2);
    tim2.cr1.modify(|_, w| w.cen().enabled());

    let tim3 = dp.TIM3;
    timers::configure_tim3(&tim3);
    tim3.cr1.modify(|_, w| w.cen().enabled());

    let tim4 = dp.TIM4;
    timers::configure_tim4(&tim4);
    tim4.cr1.modify(|_, w| w.cen().enabled());

    let tim9 = dp.TIM9;
    timers::configure_tim9(&tim9);

    pins::configure_pa0(&dp.GPIOA);
    pins::configure_pa1(&dp.GPIOA);

    let dma2 = dp.DMA2;
    dma::configure_dma2(&dma2, unsafe { INFRARED.as_ptr() });
    dma2.st[0].cr.modify(|_, w| w.en().enabled());

    let adc1 = dp.ADC1;
    adc::configure_adc(&adc1);
    adc1.cr2.modify(|_, w| w.swstart().start());

    pins::configure_motor_pins(&dp.GPIOB);

    intr::free(|cs| {
        G_TIM4.borrow(cs).replace(Some(tim4));
    });

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM4);
    }

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

    let mut state = State::FollowingLineAndAvoiding;

    loop {
        let readings = unsafe { INFRARED };
        let (front_dist, left_dist) = intr::free(|cs| {
            let dis = G_DISTANCES.borrow(cs).borrow();
            (dis.front.get_distance_cm(), dis.side.get_distance_cm())
        });
        let readings = SensorReadings {
            left_infrared: readings[0],
            right_infrared: readings[1],
            front_distance: front_dist,
            left_distance: left_dist,
        };
        robot.update_sensors(readings);
        state = state.process_state(&mut robot);
    }
}

#[interrupt]
fn TIM4() {
    let (front, side) = intr::free(|cs| {
        let tim4 = G_TIM4.borrow(cs).take().unwrap();
        let sr = &tim4.sr;
        let front = sr
            .read()
            .cc3if()
            .bit_is_set()
            .then_some(tim4.ccr3().read().ccr().bits());
        let side = sr
            .read()
            .cc4if()
            .bit_is_set()
            .then_some(tim4.ccr4().read().ccr().bits());
        sr.modify(|_, w| {
            w.cc3of()
                .clear_bit()
                .cc4of()
                .clear_bit()
                .cc3if()
                .clear_bit()
                .cc4if()
                .clear_bit()
        });
        G_TIM4.borrow(cs).replace(Some(tim4));
        (front, side)
    });
    intr::free(|cs| {
        let mut distances = G_DISTANCES.borrow(cs).borrow_mut();
        front
            .into_iter()
            .for_each(|t| distances.front.update_measurment(t));
        side.into_iter()
            .for_each(|t| distances.side.update_measurment(t));
    });
}
