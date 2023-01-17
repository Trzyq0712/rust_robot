use stm32f4::stm32f401::ADC1;

pub fn configure_adc(adc: &ADC1) {
    adc.sqr1.write(|w| w.l().bits(2)); // Perform a sequence of 2 conversions
    adc.sqr3.write(|w| unsafe {
        w.sq1().bits(0); // PA0 converted first
        w.sq2().bits(1) // PA1 converted second
    });
    adc.cr1.write(|w| {
        w.res().ten_bit(); // Set resolution to 10 bits
        w.scan().enabled() // Perform the next conversion after the previous one
    });
    adc.cr2.write(|w| {
        w.dma().enabled(); // Use DMA transfers to save the readings
        w.cont().continuous(); // After the group of channels is converted repeat
        w.adon().enabled().dds().continuous() // Turn on the ADC
    });
}
