#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m::asm::nop;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{Config, Uart};
use embassy_stm32::{bind_interrupts, peripherals, usart};
use embassy_time::Timer;
use heapless::String;
use lcd::{Delay, Display, Hardware};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART2 => usart::InterruptHandler<peripherals::USART2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let mut heater = Output::new(p.PB12, Level::High, Speed::Low);

    let scl_i2c2 = p.PB10;
    let sda_i2c2 = p.PB3;

    let i2c2 = I2c::new_blocking(p.I2C2, scl_i2c2, sda_i2c2, Hertz(100_000), Default::default());
    let mut bmp = bmp280_ehal::BMP280::new(i2c2).unwrap();

    let scl_i2c1 = p.PB6;
    let sda_i2c1 = p.PB7;
    let i2c1 = I2c::new_blocking(p.I2C1, scl_i2c1, sda_i2c1, Hertz(100_000), Default::default());

    let dev = Pcf8574::new(i2c1);
    let mut display = Display::new(dev);
    display.init(lcd::FunctionLine::Line2, lcd::FunctionDots::Dots5x8);
    display.display(
        lcd::DisplayMode::DisplayOn,
        lcd::DisplayCursor::CursorOff,
        lcd::DisplayBlink::BlinkOff,
    );

    display.clear();
    display.home();
    core::write!(&mut display, "Reading temp...").unwrap();

    let config = Config::default();
    let mut usart = Uart::new_blocking(p.USART2, p.PA3, p.PA2, config).unwrap();

    loop {
        let mut acc: f64 = 0.0;
        let mut acc_pressure: f64 = 0.0;
        for _ in 0..59 {
            acc += bmp.temp_one_shot();
            acc_pressure += bmp.pressure_one_shot();
            Timer::after_millis(1000).await;
        }
        let temp = acc / 60.0;
        let pressure = acc_pressure / 60.0;

        if temp < 23.5 {
            heater.set_low(); // Turn on the heater
        } else {
            heater.set_high(); // Turn off the heater
        }

        info!("Temperature: {}", temp);
        info!("Pressure: {}", pressure);

        // Display
        display.clear();
        display.home();
        core::write!(&mut display, "T: {:.2}C", temp).unwrap();
        display.position(0, 1);
        core::write!(&mut display, "P: {:.2}", pressure).unwrap();

        // USART
        let mut write_with_nlcr: String<128> = String::new();
        core::write!(&mut write_with_nlcr, "{{\"celsius\": {:.2}}}\r\n", temp).unwrap();
        unwrap!(usart.blocking_write(write_with_nlcr.as_bytes()));
    }
}

pub struct Pcf8574<'a> {
    dev: I2c<'a, Blocking>,
    data: u8,
}

impl<'a> Pcf8574<'a> {
    pub fn new(i2c: I2c<'a, Blocking>) -> Self {
        Self {
            dev: i2c,
            data: 0b0000_1000, // backlight on by default
        }
    }

    /// Set the display's backlight on or off.
    pub fn backlight(&mut self, on: bool) {
        self.set_bit(3, on);
        self.apply();
    }

    fn set_bit(&mut self, offset: u8, bit: bool) {
        if bit {
            self.data |= 1 << offset;
        } else {
            self.data &= !(1 << offset);
        }
    }
}

impl Delay for Pcf8574<'_> {
    fn delay_us(&mut self, _delay_usec: u32) {
        for _ in 0..1_000 {
            nop();
        }
    }
}

impl Hardware for Pcf8574<'_> {
    fn rs(&mut self, bit: bool) {
        self.set_bit(0, bit);
    }

    fn enable(&mut self, bit: bool) {
        self.set_bit(2, bit);
    }

    fn data(&mut self, bits: u8) {
        self.data = (self.data & 0x0F) | (bits << 4);
    }

    fn apply(&mut self) {
        self.dev.blocking_write(0x27, &[self.data]).unwrap();
    }
}
