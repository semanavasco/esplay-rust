//! ESPlay Micro v2 - Learning
//!
//! Initializes the ILI9341 LCD display and fills it with a solid color.
//!
//! Pin mapping: GPIO18 (SCK), GPIO23 (MOSI), GPIO5 (CS), GPIO12 (DC), GPIO2 (RST), GPIO27 (BL)

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;
use esp_hal::{
    Blocking,
    clock::CpuClock,
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    main, spi,
    spi::master::{Config, Spi},
};
use mipidsi::Builder;
use mipidsi::interface::SpiInterface;
use mipidsi::models::ILI9341Rgb565;
use mipidsi::options::{ColorOrder, Orientation};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

const HEAP_SIZE: usize = 98768;
const DISPLAY_WIDTH: u16 = 320;
const DISPLAY_HEIGHT: u16 = 240;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: HEAP_SIZE);

    // LCD pins
    let dc = Output::new(peripherals.GPIO12, Level::High, OutputConfig::default());
    let cs = Output::new(peripherals.GPIO5, Level::High, OutputConfig::default());
    let _bl = Output::new(peripherals.GPIO27, Level::High, OutputConfig::default());
    let mut rst = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // SPI setup
    let spi_bus: Spi<'_, Blocking> = spi::master::Spi::new(peripherals.SPI2, Config::default())
        .expect("Could not create SPI bus")
        .with_sck(peripherals.GPIO18)
        .with_mosi(peripherals.GPIO23);
    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi_bus, cs)
        .expect("Could not create SPI device");

    let mut buf = [0u8; 512];
    let spi_iface = SpiInterface::new(spi_device, dc, &mut buf);

    // Display initialization with hardware reset
    let mut delay = Delay::new();
    rst.set_low();
    delay.delay_millis(20u32);
    rst.set_high();
    delay.delay_millis(200u32);

    let mut display = Builder::new(ILI9341Rgb565, spi_iface)
        .orientation(Orientation::new())
        .display_size(DISPLAY_HEIGHT, DISPLAY_WIDTH)
        .color_order(ColorOrder::Bgr)
        .init(&mut delay)
        .expect("Could not initialize display");

    // Fill screen with solid color
    let total_pixels = (DISPLAY_WIDTH as u32) * (DISPLAY_HEIGHT as u32);
    let pixels = core::iter::repeat(Rgb565::BLUE).take(total_pixels as usize);
    display
        .set_pixels(0, 0, DISPLAY_HEIGHT, DISPLAY_WIDTH, pixels)
        .expect("Could not draw to display");

    loop {
        delay.delay_millis(500u32);
    }
}
