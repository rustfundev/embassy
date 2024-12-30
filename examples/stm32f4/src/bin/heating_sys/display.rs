use core::fmt::Write;
use cortex_m::asm::nop;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Blocking;
use embassy_stm32::peripherals;
use embassy_stm32::time::Hertz;
use embassy_time::Timer;
use lcd::{Delay, Display, Hardware};
use portable_atomic::{AtomicF64, Ordering};

#[embassy_executor::task]
pub async fn run_task(
    i2c1: peripherals::I2C1,
    scl_i2c1: peripherals::PB6,
    sda_i2c1: peripherals::PB7,
    temperature: &'static AtomicF64,
    pressure: &'static AtomicF64,
) {
    let i2c1 = I2c::new_blocking(i2c1, scl_i2c1, sda_i2c1, Hertz(100_000), Default::default());

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
    display.print("Reading temp...");

    loop {
        let temperature = temperature.load(Ordering::Relaxed);
        let pressure = pressure.load(Ordering::Relaxed);

        if temperature != 0.0 && pressure != 0.0 {
            display.clear();
            display.home();
            core::write!(&mut display, "T: {:.2}C", temperature).unwrap();
            display.position(0, 1);
            core::write!(&mut display, "P: {:.2}", pressure).unwrap();
        }
        Timer::after_secs(10).await;
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
