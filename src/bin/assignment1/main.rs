#![no_main]
#![no_std]
#![allow(unused)]

use core::cell::{Cell, RefCell};
use core::sync::atomic::{AtomicU32, Ordering};

// Halt on panic
use panic_halt as _; // panic handler

use cortex_m::{asm, interrupt as intr, interrupt::Mutex};
use cortex_m_rt::{entry, exception};
use stm32::interrupt;
use stm32f4::stm32f401 as stm32;

use rtt_target::{rprintln, rtt_init_print};

static G_TIM2: Mutex<RefCell<Option<stm32::TIM2>>> = Mutex::new(RefCell::new(None));
static G_INTR_COUNTER: AtomicU32 = AtomicU32::new(0);

#[entry]
fn main() -> ! {
    rtt_init_print!();
    rprintln!("Hello2");

    let dp = stm32::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    dp.RCC.ahb1enr.modify(|_, w| w.gpiocen().enabled());
    dp.RCC.apb1enr.modify(|_, w| w.tim2en().enabled());

    dp.GPIOC.moder.modify(|_, w| w.moder13().output());
    dp.GPIOC.otyper.modify(|_, w| w.ot13().push_pull());

    intr::free(|cs| {
        dp.TIM2.arr.write(|w| w.arr().bits(0xFF));
        dp.TIM2.cr1.write(|w| {
            w.urs().counter_only();
            w.arpe().enabled();
            w.cen().enabled()
        });
        dp.TIM2.dier.modify(|_, w| w.uie().enabled());
        dp.TIM2.psc.write(|w| w.psc().bits(0xFFFF));
        dp.TIM2.egr.write(|w| w.ug().update());

        G_TIM2.borrow(cs).replace(Some(dp.TIM2));
    });

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::TIM2);
    }
    loop {
        rprintln!("Loop");
        dp.GPIOC.odr.modify(|r, w| w.odr13().bit(!r.odr13().bit()));
        rprintln!(
            "TIM2 cnt: {}",
            intr::free(|cs| G_TIM2
                .borrow(cs)
                .borrow()
                .as_ref()
                .unwrap()
                .cnt
                .read()
                .bits())
        );
        rprintln!("Counter: {}", G_INTR_COUNTER.load(Ordering::Relaxed));
        asm::delay(5_000_000);
    }
}

#[interrupt]
fn TIM2() {
    rprintln!("Interrupt");
    G_INTR_COUNTER.fetch_add(1, Ordering::Relaxed);
    intr::free(|cs| {
        let mut tim2 = G_TIM2.borrow(cs).borrow_mut();
        tim2.as_mut().unwrap().sr.modify(|_, w| w.uif().clear_bit());
    });
}
