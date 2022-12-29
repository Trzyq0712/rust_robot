use stm32f4::stm32f401::{GPIOA, GPIOB};

fn configure_pb14(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder14().alternate());
    port.afrh.modify(|_, w| w.afrh14().af1());
}

fn configure_pb15(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder15().alternate());
    port.afrh.modify(|_, w| w.afrh15().af1());
}

fn configure_pb3(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder3().alternate());
    port.afrl.modify(|_, w| w.afrl3().af1());
}

fn configure_pa8(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder8().alternate());
    port.afrh.modify(|_, w| w.afrh8().af1());
}

fn configure_pb10(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder10().alternate());
    port.otyper.modify(|_, w| w.ot10().push_pull());
    port.afrh.modify(|_, w| w.afrh10().af1());
}

fn configure_pb11(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder11().alternate());
    port.afrh.modify(|_, w| w.afrh11().af1());
}

/// Configures ports for motors.
/// Left motor: pb14, pb15
/// Right motor: pa8, pb3
pub fn configure_motor_pins(gpioa: &GPIOA, gpiob: &GPIOB) {
    configure_pa8(gpioa);
    configure_pb3(gpiob);
    configure_pb14(gpiob);
    configure_pb15(gpiob);
}

/// Configure ports for leds.
/// Left led: pb10
/// Right led: pb11
pub fn configure_led_indicators(gpiob: &GPIOB) {
    configure_pb10(gpiob);
    configure_pb11(gpiob);
}
