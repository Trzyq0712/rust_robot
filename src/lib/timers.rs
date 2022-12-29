use stm32f4::stm32f401::{TIM1, TIM2};

const AUTO_RELOAD: u16 = 1 << 11;

pub fn configure_tim1(tim: &TIM1) {
    tim.arr.modify(|_, w| w.arr().bits(AUTO_RELOAD));
}

pub fn configure_tim2(tim: &TIM2) {
    tim.arr.modify(|_, w| w.arr().bits(AUTO_RELOAD as u32));
    tim.ccr.iter().for_each(|ccr| ccr.modify(|_, w| w.bits(0)));
    tim.cr1.modify(|_, w| w.cen().enabled());
    tim.ccer.modify(|_, w| w.cc3e().set_bit());
    tim.ccmr2_output().modify(|_, w| {
        w.oc3m().pwm_mode1();
        w.cc3s().output()
    })
}

/// Sets duty of the channel.
/// duty is between 0 and 100
pub fn set_duty_tim2(tim: &TIM2, channel: u8, duty: u8) {
    let max_duty = tim.arr.read().arr().bits();
    tim.ccr[channel as usize].write(|w| w.bits(max_duty * duty as u32 / 100));
}
