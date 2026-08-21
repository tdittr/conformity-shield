use super::{SharedI2c, SharedI2cMutex};
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_rp::{
    Peri,
    gpio::{
        AnyPin, Input,
        Level,
        Output, Pull,
    },
};
use embassy_time::Duration;
use ina219::{
    AsyncIna219,
    address::{Address, Pin},
    calibration::UnCalibrated,
    configuration::{
        BusVoltageRange, Configuration, MeasuredSignals, OperatingMode, Resolution,
        ShuntVoltageRange,
    },
    measurements::{BusVoltage, ShuntVoltage},
};

pub struct UsbPower {
    pub en: Output<'static>,
    pub flag: Input<'static>,
    pub ina: AsyncIna219<SharedI2c, UnCalibrated>,
    pub conversion_time: Duration,
}

impl UsbPower {
    pub async fn init(
        en: Peri<'static, AnyPin>,
        flag: Peri<'static, AnyPin>,
        shared_i2c: &'static SharedI2cMutex,
        a0: Pin,
    ) -> Self {
        let mut ina = defmt::expect!(
            AsyncIna219::new(I2cDevice::new(shared_i2c), Address::from_pins(a0, Pin::Gnd),).await,
            "Could not initialize INA219 for port {}",
            match a0 {
                Pin::Vcc => "0",
                Pin::Gnd => "1",
                _ => "oops",
            }
        );

        let conf = Configuration {
            bus_voltage_range: BusVoltageRange::Fsr16v,
            shunt_voltage_range: ShuntVoltageRange::Fsr160mv,
            bus_resolution: Resolution::Res12Bit,
            shunt_resolution: Resolution::Res12Bit,
            operating_mode: OperatingMode::Continous(MeasuredSignals::ShutAndBusVoltage),
            ..Default::default()
        };
        defmt::expect!(
            ina.set_configuration(conf).await,
            "Could not set configuration on INA219 for port {}",
            match a0 {
                Pin::Vcc => "0",
                Pin::Gnd => "1",
                _ => "oops",
            }
        );

        let conversion_time =
            Duration::from_micros(conf.conversion_time_us().unwrap_or(100).into());

        UsbPower {
            en: Output::new(en, Level::High),
            flag: Input::new(flag, Pull::Up),
            ina,
            conversion_time,
        }
    }

    pub fn enable(&mut self) {
        self.en.set_low();
    }

    pub fn disable(&mut self) {
        self.en.set_high();
    }

    pub fn status(&self) -> Result<(), ()> {
        if self.flag.is_high() { Ok(()) } else { Err(()) }
    }

    pub async fn read_meter(&mut self) -> Result<(BusVoltage, ShuntVoltage), ()> {
        let bus = self.ina.bus_voltage().await.map_err(|_| ())?;
        let shunt = self.ina.shunt_voltage().await.map_err(|_| ())?;

        Ok((bus, shunt))
    }
}
