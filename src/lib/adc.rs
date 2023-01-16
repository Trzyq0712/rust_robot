use stm32f4::stm32f401::ADC1;

pub fn configure_adc(adc: &ADC1) {
    adc.jsqr.write(|w| unsafe {
        w.jl().bits(2);
        w.jsq1().bits(0);
        w.jsq2().bits(1)
    });
    adc.cr2.write(|w| {
        w.jexten().rising_edge();
        w.jextsel().tim5trgo();
        w.adon().enabled()
    });
    adc.cr1.write(|w| {
        w.res().twelve_bit();
        w.jeocie().enabled();
        w.scan().enabled()
    });
}
