//! Dump the Anycubic Photon Mono 4k external Flash.
#![no_std]
#![no_main]

use panic_rtt_target as _;
use rtt_target::rprintln;
use spi_memory::prelude::*;
use spi_memory::series25::Flash;
use stm32f1xx_hal::prelude::*;
use stm32f1xx_hal::{pac, spi};

const FLASH_SIZE: u32 = 16 * 1024 * 1024;
const BUFFER_SIZE: usize = 32 * 1024;
const CHUNKS: usize = (FLASH_SIZE as usize) / BUFFER_SIZE;

#[cortex_m_rt::entry]
fn main() -> ! {
    // Setup RTT, one for debug and one for dumping binary.
    let channels = rtt_target::rtt_init! {
        up: {
            0: {
                size: 1024,
                mode: rtt_target::ChannelMode::NoBlockSkip,
                name: "Terminal"
            }
            1: {
                size: BUFFER_SIZE,
                mode: rtt_target::ChannelMode::BlockIfFull,
                name: "Binary"
            }
        }
    };
    let (terminal, mut binary) = channels.up;
    rtt_target::set_print_channel(terminal);
    rprintln!("INIT");

    let dp = pac::Peripherals::take().unwrap();
    // Flash memory.
    let mut flash = dp.FLASH.constrain();
    // Reset & Clock Control.
    let rcc = dp.RCC.constrain();
    // Initialize the device to run at 48Mhz using the 8Mhz crystal on
    // the PCB instead of the internal oscillator.
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(48.MHz())
        .freeze(&mut flash.acr);

    // Use GPIO B for external flash SPI access.
    let mut gpiob = dp.GPIOB.split();

    let cs = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let sck = gpiob.pb13.into_alternate_push_pull(&mut gpiob.crh);
    let miso = gpiob.pb14.into_floating_input(&mut gpiob.crh);
    let mosi = gpiob.pb15.into_alternate_push_pull(&mut gpiob.crh);

    // Configure SPI for external flash access.
    let spi = spi::Spi::spi2(
        dp.SPI2,
        (sck, miso, mosi),
        spi::Mode {
            polarity: spi::Polarity::IdleLow,
            phase: spi::Phase::CaptureOnFirstTransition,
        },
        clocks.pclk1(), // Run as fast as we can. The flash chip can go up to 133Mhz.
        clocks,
    );
    let mut ex_flash = Flash::init(spi, cs).unwrap();

    // Read the external flash chip JEDEC ID.
    let id = ex_flash.read_jedec_id().unwrap();
    rprintln!("FLASH {:?}", id);

    // Dump the external flash chip contents via RTT.
    let mut buf = [0u8; BUFFER_SIZE];
    for (addr, index) in (0..FLASH_SIZE).step_by(BUFFER_SIZE).zip(1usize..) {
        rprintln!("DUMP 0x{:08X} {} / {}", addr, index, CHUNKS);
        ex_flash.read(addr, &mut buf).unwrap();
        let written = binary.write(&buf);
        // This shouldn't happen, as the write should be blocking (?)
        if written < BUFFER_SIZE {
            panic!("Incomplete write");
        }
    }

    // Done.
    panic!("HALT");
}
