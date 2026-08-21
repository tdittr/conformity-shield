#![no_std]
#![expect(dead_code)]

use defmt::{debug, info};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    Peri, bind_interrupts,
    clocks::ClockConfig,
    gpio::{Level::Low, Output},
    i2c::{self, I2c},
    peripherals::{I2C1, PIN_16, PIN_17, PIN_18, PIN_19, UART0, USB},
    uart::{self, Async, Uart},
    usb,
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use ina219::address::Pin;
use static_cell::StaticCell;

use crate::{
    peripherals::{DutPins, OtherPeripherals, split_peripherals},
    usb_power::UsbPower,
};

type SharedI2c = I2cDevice<'static, ThreadModeRawMutex, I2c<'static, I2C1, i2c::Async>>;
type SharedI2cMutex = Mutex<ThreadModeRawMutex, I2c<'static, I2C1, i2c::Async>>;

pub mod peripherals;
pub mod selftest;

bind_interrupts!(
    pub struct ShieldIrqs {
        I2C1_IRQ => i2c::InterruptHandler<I2C1>;
        UART0_IRQ => uart::InterruptHandler<UART0>;
        USBCTRL_IRQ => usb::InterruptHandler<USB>;
    }
);

pub const XOSC_HZ: u32 = 12_000_000;

pub struct UsbHost {
    en: Output<'static>,
    sel: Output<'static>,

    dp: Peri<'static, PIN_18>,
    dn: Peri<'static, PIN_19>,
}

impl UsbHost {
    pub fn new(
        en: Peri<'static, PIN_16>,
        sel: Peri<'static, PIN_17>,
        dp: Peri<'static, PIN_18>,
        dn: Peri<'static, PIN_19>,
    ) -> Self {
        Self {
            en: Output::new(en, Low),
            sel: Output::new(sel, Low),
            dp,
            dn,
        }
    }
}

mod usb_power;

pub struct Shield {
    pub usb_power0: UsbPower,
    pub usb_power1: UsbPower,

    #[cfg(not(feature = "rev1"))]
    pub usb_host1: UsbHost,

    // pub usb: usb::Driver<'static, USB>,
    pub i2c: &'static SharedI2cMutex,
    pub debugger_uart: Uart<'static, Async>,
    pub status_led: Output<'static>,

    pub dut_pins: DutPins,
    pub other_peripherals: OtherPeripherals,
}

impl Shield {
    /// Initializes all used pins and peripherals of the shield
    ///
    /// # Panics
    /// If called twice or if [`embassy_rp::init`] was called before.
    pub async fn init() -> Self {
        let config = embassy_rp::config::Config::new(ClockConfig::crystal(XOSC_HZ));
        let p = embassy_rp::init(config);

        let (pins, dut_pins, p, other_peripherals) = split_peripherals(p);

        debug!("Setting up I2C peripherals");
        static SHARED_I2C: StaticCell<SharedI2cMutex> = StaticCell::new();
        let i2c = SHARED_I2C.init(Mutex::new(I2c::new_async(
            p.I2C1,
            pins.scl,
            pins.sda,
            ShieldIrqs,
            Default::default(),
        )));

        debug!("Setting up power port 0");
        let usb_power0 =
            UsbPower::init(pins.usb0_en.into(), pins.usb0_flag.into(), i2c, Pin::Vcc).await;

        debug!("Setting up power port 1");
        let usb_power1 =
            UsbPower::init(pins.usb1_en.into(), pins.usb1_flag.into(), i2c, Pin::Gnd).await;

        debug!("Setting up USB host");
        #[cfg(not(feature = "rev1"))]
        let usb_host1 = UsbHost::new(
            pins.usb_host_en,
            pins.usb_host_sel,
            pins.usb_host_dp,
            pins.usb_host_dn,
        );

        debug!("Setting up UART");
        let debugger_uart = Uart::new(
            p.UART0,
            pins.todi,
            pins.tido,
            ShieldIrqs,
            p.DMA_CH14,
            p.DMA_CH15,
            Default::default(),
        );

        debug!("Setting up status led");
        let status_led = Output::new(pins.led, embassy_rp::gpio::Level::High);

        // debug!("Setting up USB device");
        // let usb = usb::Driver::new(p.USB, ShieldIrqs);

        info!("Shield initialized!");

        Self {
            usb_power0,
            usb_power1,

            #[cfg(not(feature = "rev1"))]
            usb_host1,

            i2c,
            debugger_uart,
            status_led,
            // usb,
            dut_pins,
            other_peripherals,
        }
    }
}
