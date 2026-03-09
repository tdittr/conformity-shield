#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Ticker};
use panic_probe as _;
use shield_bsp::{Shield, selftest};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello world!");

    let mut shield = Shield::init().await;

    let result = selftest::test_all(&mut shield).await;

    info!("Done!");

    // cortex_m_semihosting::debug::exit(result);

    shield.usb_power0.enable();
    shield.usb_power1.enable();

    let mut ticker = Ticker::every(Duration::from_hz(1));
    loop {
        info!("Port 0: {}", shield.usb_power0.read_meter().await);
        info!("Port 1: {}", shield.usb_power1.read_meter().await);

        ticker.next().await;
    }
}
