#![no_std]
#![no_main]

use display_interface_spi::SPIInterfaceNoCS;

use embedded_graphics::{
    prelude::{RgbColor, Point, DrawTarget},
    pixelcolor::Rgb565,
    mono_font::{
        ascii::FONT_10X20,
        MonoTextStyleBuilder,
    },
    text::{Alignment, Text},
    Drawable,
};

use esp32s3_hal::{
    clock::{ClockControl, CpuClock},
    peripherals::Peripherals,
    prelude::*,
    spi,
    timer::TimerGroup,
    Rtc,
    IO,
    Delay,
};

use mipidsi::{ Orientation, ColorOrder };

#[allow(unused_imports)]
use esp_backtrace as _;

use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut wdt0 = timer_group0.wdt;
    let timer_group1 = TimerGroup::new(peripherals.TIMG1, &clocks);
    let mut wdt1 = timer_group1.wdt;

    rtc.rwdt.disable();
    wdt0.disable();
    wdt1.disable();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let sclk = io.pins.gpio7;
    let mosi = io.pins.gpio6;
    let mut backlight = io.pins.gpio45.into_push_pull_output();
    backlight.set_high().unwrap();

    let spi = spi::Spi::new_no_cs_no_miso(
        peripherals.SPI2,
        sclk,
        mosi,
        60u32.MHz(),
        spi::SpiMode::Mode0,
        &mut system.peripheral_clock_control,
        &clocks,
    );

    let di = SPIInterfaceNoCS::new(spi, io.pins.gpio4.into_push_pull_output());
    let reset = io.pins.gpio48.into_push_pull_output();

    let mut delay = Delay::new(&clocks);

    let mut display = mipidsi::Builder::ili9342c_rgb565(di)
        .with_display_size(320, 240)
        .with_orientation(Orientation::PortraitInverted(false))
        .with_color_order(ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
        .unwrap();

    display.clear(Rgb565::WHITE).unwrap();

    let default_style = MonoTextStyleBuilder::new()
        .font(&FONT_10X20)
        .text_color(RgbColor::BLACK)
        .build();

    let mut y;

    for b in 0..2 {
        for a in (1..516).step_by(8) {
            for x in (1..400).step_by(45) {
                y = a + (a / a + 11) * x;
                if b == 0 {
                    Text::with_alignment("o", Point::new(x, y), default_style,  Alignment::Center)
                        .draw(&mut display)
                        .unwrap();
                    Text::with_alignment("o", Point::new(y, x + 8), default_style,  Alignment::Center)
                        .draw(&mut display)
                        .unwrap();
                } else {
                    Text::with_alignment("o", Point::new(x + 25, y + 43), default_style,  Alignment::Center)
                        .draw(&mut display)
                        .unwrap();
                    Text::with_alignment("o", Point::new(y + 25, x + 31), default_style,  Alignment::Center)
                        .draw(&mut display)
                        .unwrap();
                }
            }
        }
    }
    loop{}
}