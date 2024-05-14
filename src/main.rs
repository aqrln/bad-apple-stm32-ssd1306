#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _};

use display_interface::DisplayError;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Pin, Speed},
    i2c::I2c,
    time::Hertz,
};
use embassy_time::{Duration, Instant, Timer};
use embedded_graphics::{pixelcolor::BinaryColor, prelude::*};
use ssd1306::{
    mode::DisplayConfig, prelude::Brightness, rotation::DisplayRotation, size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};
use tinygif::Gif;

use defmt::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    info!("started!");
    spawner.spawn(startup_blinky(p.PC13)).unwrap();

    let i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, Hertz(400_000), Default::default());
    let di = I2CDisplayInterface::new(i2c);
    graphics(di).await.unwrap();
}

#[embassy_executor::task]
async fn startup_blinky(led_pin: impl Pin) {
    let mut led_out = Output::new(led_pin, Level::High, Speed::Low);

    for _ in 0..5 {
        led_out.toggle();
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[allow(dead_code)] // the warning is wrong: unwrap() can print the fields
#[derive(Debug)]
enum VideoError {
    DisplayError(DisplayError),
    GifError(tinygif::ParseError),
}

impl From<DisplayError> for VideoError {
    fn from(e: DisplayError) -> Self {
        VideoError::DisplayError(e)
    }
}

impl From<tinygif::ParseError> for VideoError {
    fn from(e: tinygif::ParseError) -> Self {
        VideoError::GifError(e)
    }
}

async fn graphics(
    interface: impl display_interface::WriteOnlyDataCommand,
) -> Result<(), VideoError> {
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();

    display.init()?;
    display.clear_buffer();
    display.flush()?;
    display.set_brightness(Brightness::NORMAL)?;

    let image = Gif::<BinaryColor>::from_slice(include_bytes!("../bad-apple.gif"))?;

    for (i, frame) in image.frames().enumerate() {
        let frame_begin = Instant::now();

        frame.draw(&mut display.translated(Point::new(20, 0)))?;
        display.flush()?;

        let frame_duration = frame_begin.elapsed();
        info!("rendered frame {} in {:?}us", i, frame_duration.as_micros());

        Timer::after(Duration::from_millis(125) - frame_duration).await;
    }

    Ok(())
}

#[embassy_executor::task]
async fn audio_task(audio_pin: AnyPin, mut_btn_pin: AnyPin) {
    let mut out = Output::new(audio_pin, Level::Low, Speed::Low);
    let mute_btn = Input::new(mut_btn_pin, Pull::Up);

    loop {
        if mute_btn.is_low() {
            break;
        }

        out.toggle();
        Timer::after(Duration::from_micros(2273)).await;
    }
}
