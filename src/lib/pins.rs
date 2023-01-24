use stm32f4::stm32f401::{GPIOA, GPIOB};

/// Configure to drive the left motor forward.
/// Uses TIM3 CH3.
fn configure_pb0(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder0().alternate());
    port.afrl.modify(|_, w| w.afrl0().af2());
}

/// Configure to drive the left motor backward.
/// Uses TIM3 CH4.
fn configure_pb1(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder1().alternate());
    port.afrl.modify(|_, w| w.afrl1().af2());
}

/// Configure to drive the right motor forward.
/// Uses TIM3 CH1.
fn configure_pb4(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder4().alternate());
    port.afrl.modify(|_, w| w.afrl4().af2());
}

/// Configure to drive the right motor backward.
/// Uses TIM3 CH2.
fn configure_pb5(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder5().alternate());
    port.afrl.modify(|_, w| w.afrl5().af2());
}

/// Configures pins for motors.
/// Left motor: pb0, pb1
/// Right motor: pb4, pb5
pub fn configure_motor_pins(gpiob: &GPIOB) {
    configure_pb0(gpiob);
    configure_pb1(gpiob);
    configure_pb4(gpiob);
    configure_pb5(gpiob);
}

/// Configure to be the trigger for the front ultrasound.
/// Uses TIM9 CH1.
pub fn configure_pa2(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder2().alternate());
    port.afrl.modify(|_, w| w.afrl2().af3());
}

/// Configure to be the trigger for the side ultrasound.
/// Uses TIM9 CH2.
pub fn configure_pa3(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder3().alternate());
    port.afrl.modify(|_, w| w.afrl3().af3());
}

/// Configure to be the echo for the side ultrasound.
/// Uses TIM4 CH3.
pub fn configure_pb8(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder8().alternate());
    port.afrh.modify(|_, w| w.afrh8().af2());
}

/// Configure to be the echo for the side ultrasound.
/// Uses TIM4 CH4.
pub fn configure_pb9(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder9().alternate());
    port.afrh.modify(|_, w| w.afrh9().af2());
}

pub fn configure_ultrasound_pins(porta: &GPIOA, portb: &GPIOB) {
    configure_pa2(porta);
    configure_pa3(porta);
    configure_pb8(portb);
    configure_pb9(portb);
}

/// Configure pin for the left infrared sensor.
/// Uses ADC IN4.
pub fn configure_pa4(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder4().analog());
}

/// Configure pin for the left infrared sensor.
/// Uses ADC IN5.
pub fn configure_pa5(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder5().analog());
}

/// Configure to be the left speed encoder.
/// Uses TIM2 CH1.
pub fn configure_pa0(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder0().alternate());
    port.afrl.modify(|_, w| w.afrl0().af1());
}

/// Configure to be the left speed encoder.
/// Uses TIM5 CH2.
pub fn configure_pa1(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder1().alternate());
    port.afrl.modify(|_, w| w.afrl1().af2());
}
