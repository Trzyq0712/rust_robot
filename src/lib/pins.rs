use stm32f4::stm32f401::{GPIOA, GPIOB};

/// Used to drive the left motor forward.
/// Uses TIM3 CH3.
fn configure_pb0(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder0().alternate());
    port.afrl.modify(|_, w| w.afrl0().af2());
}

/// Used to drive the left motor backward.
/// Uses TIM3 CH4.
fn configure_pb1(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder1().alternate());
    port.afrl.modify(|_, w| w.afrl1().af2());
}

/// Used to drive the right motor forward.
/// Uses TIM3 CH1.
fn configure_pb4(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder4().alternate());
    port.afrl.modify(|_, w| w.afrl4().af2());
}

/// Used to drive the right motor backward.
/// Uses TIM3 CH2.
fn configure_pb5(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder5().alternate());
    port.afrl.modify(|_, w| w.afrl5().af2());
}

// Pin A10 shall serve as the trigger to the proximity sensor.
// TIM1 CH3 controls the output.
pub fn configure_pa10(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder10().alternate());
    // port.pupdr.modify(|_, w| w.pupdr11().pull_up());
    port.afrh.modify(|_, w| w.afrh10().af1());
}

fn configure_pb10(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder10().alternate());
    port.afrh.modify(|_, w| w.afrh10().af1());
}

fn configure_pb11(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder11().alternate());
    port.afrh.modify(|_, w| w.afrh11().af1());
}

pub fn configure_pa2(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder2().alternate());
    //port.pupdr.modify(|_, w| w.pupdr2().pull_up());
    port.afrl.modify(|_, w| w.afrl2().af3());
}
pub fn configure_pa3(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder3().alternate());
    // port.pupdr.modify(|_, w| w.pupdr3().pull_up());
    port.afrl.modify(|_, w| w.afrl3().af3());
}

pub fn configure_pb8(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder8().alternate());
    port.afrh.modify(|_, w| w.afrh8().af2());
}

pub fn configure_pb9(port: &GPIOB) {
    port.moder.modify(|_, w| w.moder9().alternate());
    port.afrh.modify(|_, w| w.afrh9().af2());
}
/// Configures ports for motors.
/// Left motor: pb0, pb1
/// Right motor: pb4, pb5
pub fn configure_motor_pins(gpiob: &GPIOB) {
    configure_pb0(gpiob);
    configure_pb1(gpiob);
    configure_pb4(gpiob);
    configure_pb5(gpiob);
}

/// Configure ports for leds.
/// Left led: pb10
/// Right led: pb11
pub fn configure_led_indicators(gpiob: &GPIOB) {
    configure_pb10(gpiob);
    configure_pb11(gpiob);
}

pub fn configure_pa0(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder0().analog());
}

pub fn configure_pa1(port: &GPIOA) {
    port.moder.modify(|_, w| w.moder1().analog());
}
