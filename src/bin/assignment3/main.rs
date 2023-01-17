#![no_main]
#![no_std]

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

    unsafe {
        stm32::NVIC::unmask(stm32::interrupt::ADC);
    }

    const LEFT_MOTOR_DUTY: u16 = (u16::MAX as f32) as u16;
    // adjust max duty to go straight
    const RIGHT_MOTOR_DUTY: u16 = (u16::MAX as f32 * 0.82) as u16;

    loop {
        let [left, right] = unsafe { INFRARED };
        use timers::Direction::*;
        let is_black = (left > 500, right > 500);
        let (left_dir, right_dir) = match is_black {
            (true, false) => (Forward, Backward),
            (false, true) => (Backward, Forward),
            _ => (Forward, Forward),
        };
        timers::set_left_motor_duty(
            &tim3,
            if left_dir == Forward {
                LEFT_MOTOR_DUTY / 3 * 2
            } else {
                LEFT_MOTOR_DUTY
            },
            left_dir,
        );
        timers::set_right_motor_duty(
            &tim3,
            if right_dir == Forward {
                RIGHT_MOTOR_DUTY / 3 * 2
            } else {
                RIGHT_MOTOR_DUTY
            },
            right_dir,
        );
        rprintln!("Dr: {}", adc1.dr.read().bits());
        rprintln!("NDTR: {}", dma2.st[0].ndtr.read().bits());
        rprintln!("Error: {}", dma2.lisr.read().dmeif0().is_error());
        rprintln!("Left reading: {}, Right reading: {}", left, right);
        rprintln!("Left dir: {:?}, Right dir: {:?}", left_dir, right_dir);
        asm::delay(10_000_000);
        //        let diff = u16::abs_diff(left, right);
        //
        //        let mut duties = (LEFT_MOTOR_DUTY, RIGHT_MOTOR_DUTY);
        //        let mut directions = (Forward, Forward);
        //
        //        match diff {
        //            0..=400 => {}
        //            401..=600 => duties = (duties.0 / 3 * 2, duties.1),
        //            601..=1000 => duties = (duties.0 / 3, duties.1),
        //            1001..=1500 => duties = (0, duties.1),
        //            1501..=u16::MAX => directions = (Backward, Forward),
        //        };
        //
        //        if left > right {
        //            mem::swap(&mut duties.0, &mut duties.1);
        //            mem::swap(&mut directions.0, &mut directions.1);
        //        }
        //        // asm::delay(10_000_000);
        //        timers::set_left_motor_duty(&tim3, duties.0, directions.0);
        //        timers::set_right_motor_duty(&tim3, duties.1, directions.1);
    }
}

// #[interrupt]
// fn ADC() {
//     let readings = intr::free(|cs| {
//         let adc = G_ADC1.borrow(cs).borrow_mut();
//         let left = adc.as_ref().unwrap().jdr1().read().jdata().bits();
//         let right = adc.as_ref().unwrap().jdr2().read().jdata().bits();
//         adc.as_ref().unwrap().sr.modify(|_, w| w.jeoc().clear_bit());
//         (left, right)
//     });
//     intr::free(|cs| G_INFRARED.borrow(cs).set(readings));
// }
