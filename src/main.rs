#![no_std]
#![no_main]

mod fmt;

use display_interface::DisplayError;
use embedded_graphics::{
    geometry::{Point, Size},
    mono_font::{self, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    primitives::{Primitive, PrimitiveStyleBuilder, Rectangle},
    text::{Alignment, Baseline, Text, TextStyleBuilder},
    Drawable,
};
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Pin, Speed},
    i2c::I2c,
    time::Hertz,
};
use embassy_time::{Duration, Timer};
use ssd1306::{
    mode::DisplayConfig, prelude::Brightness, rotation::DisplayRotation, size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};

use fmt::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("started!");
    spawner.spawn(startup_blinky(p.PC13)).unwrap();

    let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, Hertz(400_000), Default::default());
    let di = I2CDisplayInterface::new(i2c);
    graphics(di).unwrap();
}

#[embassy_executor::task]
async fn startup_blinky(led_pin: impl Pin) {
    let mut led_out = Output::new(led_pin, Level::High, Speed::Low);

    for _ in 0..5 {
        led_out.toggle();
        Timer::after(Duration::from_millis(100)).await;
    }
}

fn graphics(interface: impl display_interface::WriteOnlyDataCommand) -> Result<(), DisplayError> {
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init()?;
    display.clear_buffer();
    display.flush()?;
    display.set_brightness(Brightness::NORMAL)?;

    let style = PrimitiveStyleBuilder::new()
        .stroke_color(BinaryColor::On)
        .stroke_width(1)
        .fill_color(BinaryColor::Off)
        .build();

    Rectangle::new(Point::new(1, 1), Size::new(126, 62))
        .into_styled(style)
        .draw(&mut display)?;

    let character_style = MonoTextStyleBuilder::new()
        .font(&mono_font::ascii::FONT_9X15_BOLD)
        .text_color(BinaryColor::On)
        .build();

    let text_style = TextStyleBuilder::new()
        .baseline(Baseline::Middle)
        .alignment(Alignment::Center)
        .build();

    Text::with_text_style("UwU", Point::new(64, 32), character_style, text_style)
        .draw(&mut display)?;

    display.flush()?;

    Ok(())
}
