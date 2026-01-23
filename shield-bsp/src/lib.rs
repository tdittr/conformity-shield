#![no_std]
#![expect(unused_variables, dead_code)]

use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    Peri, bind_interrupts,
    gpio::{AnyPin, Input, Level::Low, Output, Pull},
    i2c::{self, I2c},
    peripherals::{I2C1, PIN_16, PIN_17, PIN_18, PIN_19},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use ina219::{
    AsyncIna219,
    address::{Address, Pin},
    calibration::UnCalibrated,
};
use static_cell::StaticCell;

use crate::pins::ShieldPins;

type SharedI2c = I2cDevice<'static, ThreadModeRawMutex, I2c<'static, I2C1, i2c::Async>>;
type SharedI2cMutex = Mutex<ThreadModeRawMutex, I2c<'static, I2C1, i2c::Async>>;

pub mod pins;

bind_interrupts!(
    struct ShieldIrqs {
        I2C1_IRQ => i2c::InterruptHandler<I2C1>;
    }
);

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

pub struct UsbPower {
    en: Output<'static>,
    flag: Input<'static>,
    ina: AsyncIna219<SharedI2c, UnCalibrated>,
}

impl UsbPower {
    pub async fn init(
        en: Peri<'static, AnyPin>,
        flag: Peri<'static, AnyPin>,
        shared_i2c: &'static SharedI2cMutex,
        a0: Pin,
    ) -> Self {
        UsbPower {
            en: Output::new(en, Low),
            flag: Input::new(flag, Pull::Up),
            ina: defmt::expect!(
                AsyncIna219::new(I2cDevice::new(shared_i2c), Address::from_pins(a0, Pin::Gnd),)
                    .await,
                "Could not initialize INA219 for port {}",
                match a0 {
                    Pin::Vcc => 0,
                    Pin::Gnd => 1,
                    Pin::Sda => 2,
                    Pin::Scl => 3,
                }
            ),
        }
    }
}

pub struct Shield {
    usb_power0: UsbPower,
    usb_power1: UsbPower,
    usb_host0: (), // Not connected
    usb_host1: UsbHost,

    shared_i2c: &'static SharedI2cMutex,
}

impl Shield {
    pub async fn new(pins: ShieldPins, i2c: Peri<'static, I2C1>) -> Self {
        let ShieldPins {
            usb_host_en,
            usb_host_sel,
            usb_host_dp,
            usb_host_dn,
            usb0_en,
            usb0_flag,
            usb1_en,
            usb1_flag,
            todi,
            tido,
            sda,
            scl,
            led,
        } = pins;

        static SHARED_I2C: StaticCell<SharedI2cMutex> = StaticCell::new();
        let shared_i2c = SHARED_I2C.init(Mutex::new(I2c::new_async(
            i2c,
            scl,
            sda,
            ShieldIrqs,
            Default::default(),
        )));

        let usb_power0 =
            UsbPower::init(usb0_en.into(), usb0_flag.into(), shared_i2c, Pin::Vcc).await;
        let usb_power1 =
            UsbPower::init(usb1_en.into(), usb1_flag.into(), shared_i2c, Pin::Gnd).await;

        Self {
            usb_power0,
            usb_power1,
            usb_host0: (),
            usb_host1: UsbHost::new(usb_host_en, usb_host_sel, usb_host_dp, usb_host_dn),

            shared_i2c,
        }
    }

    pub fn i2c(&self) -> &'static SharedI2cMutex {
        self.shared_i2c
    }
}
