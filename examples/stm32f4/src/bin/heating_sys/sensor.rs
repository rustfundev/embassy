use defmt::*;
use embassy_stm32::i2c::I2c;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use portable_atomic::{AtomicBool, AtomicF64, Ordering};

#[embassy_executor::task]
pub async fn run_task(
    i2c2: peripherals::I2C2,
    scl_i2c2: peripherals::PB10,
    sda_i2c2: peripherals::PB3,
    temp: &'static AtomicF64,
    press: &'static AtomicF64,
    heater_on: &'static AtomicBool,
) {
    let i2c2 = I2c::new_blocking(i2c2, scl_i2c2, sda_i2c2, Hertz(100_000), Default::default());
    let mut bmp = bmp280_ehal::BMP280::new(i2c2).unwrap();

    let mut temperature;
    let mut pressure;

    info!("Reading temperature and pressure...");

    loop {
        let mut acc: f64 = 0.0;
        let mut acc_pressure: f64 = 0.0;
        for _ in 0..59 {
            acc = acc + bmp.temp_one_shot();
            acc_pressure += bmp.pressure_one_shot();
            Timer::after_millis(1000).await;
        }
        temperature = acc / 60.0;
        pressure = acc_pressure / 60.0;

        temp.store(temperature, Ordering::Relaxed);
        press.store(pressure, Ordering::Relaxed);

        match temperature {
            18.0..22.00 => {
                heater_on.store(true, Ordering::Relaxed);
            }
            22.5.. => {
                heater_on.store(false, Ordering::Relaxed);
            }
            _ => (),
        }

        info!("Temperature: {}, Pressure: {}", temperature, pressure);
    }
}
