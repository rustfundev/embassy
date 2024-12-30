use defmt::*;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::Pull;
use embassy_stm32::peripherals;
use embassy_time::Timer;
use portable_atomic::{AtomicBool, Ordering};

#[embassy_executor::task]
pub async fn run_task(button_pin: peripherals::PA0, extio: peripherals::EXTI0, heater_on: &'static AtomicBool) {
    let button = ExtiInput::new(button_pin, extio, Pull::Down);

    info!("Press the USER button...");

    loop {
        if button.is_high() {
            if !heater_on.load(Ordering::Relaxed) {
                heater_on.store(true, Ordering::Relaxed);
                info!("Power heater on");
            } else {
                heater_on.store(false, Ordering::Relaxed);
                info!("Power heater off");
            }
        }
        Timer::after_secs(3).await;
    }
}
