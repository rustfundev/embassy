use defmt::*;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::peripherals;
use embassy_time::Timer;
use portable_atomic::{AtomicBool, Ordering};

#[embassy_executor::task]
pub async fn run_task(heater_pin: peripherals::PB12, heater_on: &'static AtomicBool) {
    let mut heater = Output::new(heater_pin, Level::Low, Speed::Low);

    info!("Heater control...");

    loop {
        // Heater
        let power_on = heater_on.load(Ordering::Relaxed);
        if power_on {
            heater.set_high();
        } else {
            heater.set_low();
        }
        Timer::after_millis(1000).await;
    }
}
