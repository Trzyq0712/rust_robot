use stm32f4::stm32f401::{GPIOA, GPIOB};

pub fn configure_pb14(port: &mut GPIOB) {
    port.moder.modify(|_, w| w.moder14().alternate());
    port.afrh.modify(|_, w| w.afrh14().af1());
}

pub fn configure_pb15(port: &mut GPIOB) {
    port.moder.modify(|_, w| w.moder15().alternate());
    port.afrh.modify(|_, w| w.afrh15().af1());
}

pub fn configure_pb3(port: &mut GPIOB) {
    port.moder.modify(|_, w| w.moder3().alternate());
    port.afrl.modify(|_, w| w.afrl3().af1());
}

pub fn configure_pa8(port: &mut GPIOA) {
    port.moder.modify(|_, w| w.moder8().alternate());
    port.afrh.modify(|_, w| w.afrh8().af1());
}

/// Configures ports for motors.
/// Left motor: pb14, pb15
/// Right motor: pa8, pb3
pub fn configure_motor_pins(gpioa: &mut GPIOA, gpiob: &mut GPIOB) {
    configure_pa8(gpioa);
    configure_pb3(gpiob);
    configure_pb14(gpiob);
    configure_pb15(gpiob);
}
