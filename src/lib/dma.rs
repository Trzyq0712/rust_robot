use stm32f4::stm32f401::{ADC1, DMA2};

pub fn configure_dma2(dma: &DMA2, base_mem: *const u16) {
    dma.st[0].cr.write(|w| w.en().disabled());
    dma.st[0].cr.write(|w| {
        w.pfctrl().dma(); // The DMA will control the flow of the data
        w.dir().peripheral_to_memory(); // Copy from peripheral to memory
        w.chsel().bits(0); // Use ADC1 for Stream0
        w.psize().bits16(); // Transfer 16 bits from the peripheral
        w.msize().bits16(); // Save 16 bits in memory
        w.minc().incremented(); // We want it to increment the pointer after each transfer
        w.circ().enabled(); // We want it to continuosly copy over the 2 ADC readings
        w.pl().high()
    });
    dma.st[0]
        .m0ar
        .write(|w| unsafe { w.m0a().bits(base_mem as u32) });
    dma.st[0]
        .par
        .write(|w| unsafe { w.pa().bits((*ADC1::PTR).dr.as_ptr() as u32) });
    dma.st[0].ndtr.write(|w| w.ndt().bits(2)); // Tranfer 2 readings
}
