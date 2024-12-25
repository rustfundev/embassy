#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::i2c::I2c;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let scl = p.PA9;
    let sda = p.PA10;

    let i2c = I2c::new_blocking(p.I2C2, scl, sda, Hertz(100_000), Default::default());
    let mut bmp = bmp280_ehal::BMP280::new(i2c).unwrap();

    loop {
        let pres = bmp.pressure_one_shot();
        let temp = bmp.temp_one_shot();

        info!("Pressure: {}", pres);
        info!("Temperature: {}", temp);

        Timer::after_millis(1000).await;
    }
}
