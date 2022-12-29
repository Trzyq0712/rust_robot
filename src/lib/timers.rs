use stm32f4::stm32f401::{TIM1, TIM2, TIM4, TIM5};

const AUTO_RELOAD: u16 = 1 << 11;

// the internal clock is running at 16MHz

// configure tim1 to serve as a trigger for the proximity sensor
pub fn configure_tim1(tim: &TIM1) {
    // we need to generate a pulse of 10us (1e-5s)
    // 16e6 / 1e5 = 16
    // the arr should be at least 16, let us set it to 32 to be on the safe side
    tim.cr1.write(|w| w.opm().enabled()); // enable one-pulse mode
    tim.smcr.write(|w| {
        w.sms().trigger_mode(); // trigger the counting on the rising edge
        w.ts().itr0() // use tim5 as the trigger
    });
    tim.ccmr1_input().write(|w| w.cc1s().trc()); // use the internal event as the trigger
    tim.arr.write(|w| w.arr().bits(1 << 5)); // set the length of the pulse
    tim.ccr3().write(|w| w.ccr().bits(0)); // we do not want a delay before the pulse
    tim.ccmr2_output().write(|w| w.oc3m().pwm_mode2());
}

// configure tim2 to serve as a pwm for modulated output for peripherals
pub fn configure_tim2(tim: &TIM2) {
    tim.arr.modify(|_, w| w.arr().bits(AUTO_RELOAD as u32));
    tim.ccr.iter().for_each(|ccr| ccr.modify(|_, w| w.bits(0)));
    tim.ccer.modify(|_, w| w.cc3e().set_bit());
    tim.ccmr2_output().modify(|_, w| {
        w.oc3m().pwm_mode1();
        w.cc3s().output()
    })
}

// configure tim4 to measure the pulse lenghts of the proximity sensors
pub fn configure_tim4(tim: &TIM4) {
    tim.psc.write(|w| w.psc().bits(7));
    tim.cr2.write(|w| w.ti1s().xor()); // use CH3 as TI1
    tim.ccmr1_input().write(|w| {
        w.cc1s().ti1(); // TI1 to supply the CC1
        w.ic1f().bits(0) // not doing any sample filtering
    });
    tim.smcr.write(|w| {
        w.ts().ti1fp1();
        w.sms().reset_mode()
    });
    tim.ccmr2_input().write(|w| w.cc4s().ti3()); // select TI3 for the CC3
    tim.ccer.write(|w| {
        // detect a rising edge transition to start counting
        w.cc1p().clear_bit();
        w.cc1np().clear_bit();
        // detect a falling edge transition to stop counting
        w.cc3p().set_bit();
        w.cc3np().clear_bit()
    });
    tim.dier.write(|w| w.cc3ie().enabled()); // enable interrupts on capture
}

// configure tim5 to trigger the sending of triggers to the proximity sensors (by tim1)
pub fn configure_tim5(tim: &TIM5) {
    tim.arr.write(|w| w.arr().bits(16_000_000 * 2)); // should trigger tim1 every 2 seconds
    tim.cr2.write(|w| w.mms().update()); // send the trigger on update event
}
/// Sets duty of the channel.
/// duty is between 0 and 100
pub fn set_duty_tim2(tim: &TIM2, channel: u8, duty: u8) {
    let max_duty = tim.arr.read().arr().bits();
    tim.ccr[channel as usize].write(|w| w.bits(max_duty * duty as u32 / 100));
}
