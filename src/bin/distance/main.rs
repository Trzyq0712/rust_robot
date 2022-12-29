#![no_main]
#![no_std]
#![allow(unused)]

use core::borrow::BorrowMut;
use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

use my_hal::{pins, timers};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::{asm, interrupt as intr, interrupt::Mutex};
use cortex_m_rt::{entry, exception};
use stm32::interrupt;
use stm32f4::{stm32f401 as stm32, Reg};

use rtt_target::{rprintln, rtt_init_print};

struct DistMes {
    tim: stm32::TIM4,
    distance: u32,
}

static G_DIST_MES: Mutex<RefCell<Option<DistMes>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello");

    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC;
    rcc.ahb1enr.write(|w| {
        w.gpioaen().enabled();
        w.gpioben().enabled()
    });
    rcc.apb1enr.write(|w| {
        w.tim4en().enabled();
        w.tim5en().enabled()
    });
    rcc.apb2enr.write(|w| w.tim1en().enabled());

    let tim1 = dp.TIM1;
    timers::configure_tim1(&tim1);

    let tim4 = dp.TIM4;
    timers::configure_tim4(&tim4);

    let tim5 = dp.TIM5;
    timers::configure_tim5(&tim5);
    tim5.cr1.modify(|_, w| w.cen().enabled());

    let dist_mes = DistMes {
        tim: tim4,
        distance: Default::default(),
    };

    intr::free(|cs| G_DIST_MES.borrow(cs).replace(Some(dist_mes)));

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM4);
    }

    loop {
        rprintln!("Loop");
        let d = intr::free(|cs| G_DIST_MES.borrow(cs).borrow().as_ref().unwrap().distance);
        rprintln!("Distance is {}", d);
        asm::delay(10_000_000);
    }
}

#[interrupt]
fn TIM4() {
    rprintln!("Interrupt");
    intr::free(|cs| {
        let mut dist_mes = G_DIST_MES.borrow(cs).take().unwrap();
        let measurement = dist_mes.tim.ccr3().read().ccr().bits();
        dist_mes.tim.sr.modify(|_, w| w.cc3if().clear());
        let dist_cm = measurement as u32 * 8 / 58;
        G_DIST_MES.borrow(cs).replace(Some(DistMes {
            tim: dist_mes.tim,
            distance: dist_cm,
        }));
    });
}
