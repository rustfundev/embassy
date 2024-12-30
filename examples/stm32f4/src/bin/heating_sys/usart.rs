use core::fmt::Write;
use defmt::*;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::Timer;
use heapless::String;
use portable_atomic::{AtomicF64, Ordering};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

#[embassy_executor::task]
pub async fn run_task(
    usart2: peripherals::USART2,
    rx_pin: peripherals::PA3,
    tx_pin: peripherals::PA2,
    dma1_ch6: peripherals::DMA1_CH6,
    dma1_ch5: peripherals::DMA1_CH5,
    temperature: &'static AtomicF64,
    pressure: &'static AtomicF64,
) {
    let config = Config::default();
    let mut usart = Uart::new(usart2, rx_pin, tx_pin, Irqs, dma1_ch6, dma1_ch5, config).unwrap();

    info!("USART task...");

    loop {
        let temperature = temperature.load(Ordering::Relaxed);
        let pressure = pressure.load(Ordering::Relaxed);

        // USART
        let mut write_with_nlcr: String<40> = String::new();
        core::write!(
            &mut write_with_nlcr,
            "{{\"celsius\": {:.2}, \"hPa\": {:.2}}}\r\n",
            temperature,
            pressure
        )
        .unwrap();
        unwrap!(usart.write(write_with_nlcr.as_bytes()).await);
        Timer::after_secs(30).await;
    }
}
