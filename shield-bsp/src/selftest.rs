use crate::{Shield, peripherals::NamedPin, usb_power::UsbPower};
use defmt::{error, info, trace};
use embassy_rp::{
    Peri,
    gpio::{AnyPin, Flex, Pin, Pull},
};
use embassy_time::{Duration, Timer};

pub async fn test_not_stuck(mut pin: Flex<'_>, delay: Duration) -> Result<(), &'static str> {
    pin.set_as_input();

    let mut stuck_low = false;
    let mut stuck_high = false;

    for _ in 0..5 {
        pin.set_pull(Pull::Down);
        Timer::after(delay).await;
        stuck_low |= pin.is_high();

        pin.set_pull(Pull::Up);
        Timer::after(delay).await;
        stuck_high |= pin.is_low();
    }

    match (stuck_high, stuck_low) {
        (true, true) => Err("Weirdly sticky"),
        (true, false) => Err("Stuck high"),
        (false, true) => Err("Stuck low"),
        (false, false) => Ok(()),
    }
}

pub async fn test_not_interconnected(pins: &mut [Flex<'_>], delay: Duration) -> Result<(), ()> {
    for pin in pins.iter_mut() {
        pin.set_as_input();
        pin.set_pull(Pull::Down);
    }

    let mut has_short = false;

    for i in 0..pins.len() {
        let pin = &mut pins[i];
        pin.set_as_output();
        pin.set_high();

        Timer::after(delay).await;

        for (idx, pin) in pins.iter_mut().enumerate() {
            if idx == i {
                continue;
            }

            if pin.is_high() {
                has_short = true;
                error!("Short between pins {} and {}", i, idx);
            }
        }

        Timer::after(delay).await;

        pins[i].set_as_input();
    }

    if has_short { Err(()) } else { Ok(()) }
}

pub async fn test_power_switch(u: &mut UsbPower) -> Result<(), ()> {
    u.disable();
    Timer::after_millis(100).await;

    let bus_voltage = match u.ina.bus_voltage().await {
        Ok(bv) => bv,
        Err(_) => {
            error!("Could not read bus voltage");
            return Err(());
        }
    };

    if bus_voltage.voltage_mv() > 8 {
        error!(
            "Bus voltage still high ({} mv) even though power is switched off!",
            bus_voltage.voltage_mv()
        );
        return Err(());
    }

    u.enable();
    Timer::after_millis(10).await;

    let bus_voltage = match u.ina.bus_voltage().await {
        Ok(bv) => bv,
        Err(_) => {
            error!("Could not read bus voltage");
            return Err(());
        }
    };

    if bus_voltage.voltage_mv() < 4_700 {
        error!(
            "Bus voltage still low after enabeling power: {}",
            bus_voltage
        );
        return Err(());
    }

    let shunt_voltage = match u.ina.shunt_voltage().await {
        Ok(bv) => bv,
        Err(_) => {
            error!("Could not read shunt voltage");
            return Err(());
        }
    };

    if shunt_voltage.shunt_voltage_uv().abs() > 1000 {
        error!(
            "Shunt voltage to high after enabeling power: {}",
            shunt_voltage
        );
        return Err(());
    }

    defmt::info!(
        "USB power is ok: Bus Voltage: {}, Shunt Voltage: {}",
        bus_voltage,
        shunt_voltage
    );

    u.disable();

    Ok(())
}

pub async fn test_all(shield: &mut Shield) -> Result<(), ()> {
    let mut errors = false;

    info!("Testing if pins are stuck");

    let mut pins: [Peri<'_, AnyPin>; _] = [
        shield.dut_pins.a0.reborrow().into(),
        shield.dut_pins.a1.reborrow().into(),
        shield.dut_pins.a2.reborrow().into(),
        shield.dut_pins.a3.reborrow().into(),
        shield.dut_pins.a4.reborrow().into(),
        shield.dut_pins.a5.reborrow().into(),
        shield.dut_pins.a6.reborrow().into(),
        shield.dut_pins.d00.reborrow().into(),
        shield.dut_pins.d01.reborrow().into(),
        shield.dut_pins.d02.reborrow().into(),
        shield.dut_pins.d03.reborrow().into(),
        shield.dut_pins.d04.reborrow().into(),
        shield.dut_pins.d05.reborrow().into(),
        shield.dut_pins.d06.reborrow().into(),
        shield.dut_pins.d07.reborrow().into(),
        shield.dut_pins.d08.reborrow().into(),
        shield.dut_pins.d09.reborrow().into(),
        shield.dut_pins.d10.reborrow().into(),
        shield.dut_pins.d11.reborrow().into(),
        shield.dut_pins.d12.reborrow().into(),
        shield.dut_pins.d13.reborrow().into(),
        shield.dut_pins.d14.reborrow().into(),
        shield.dut_pins.d15.reborrow().into(),
        shield.dut_pins.d10_alt.reborrow().into(),
        shield.dut_pins.d11_alt.reborrow().into(),
        shield.dut_pins.d12_alt.reborrow().into(),
        shield.dut_pins.d13_alt.reborrow().into(),
    ];

    for pin in &mut pins {
        trace!("Checking if pin {} is stuck", NamedPin(&**pin));

        let delay = if pin.pin() >= 40 {
            Duration::from_millis(10)
        } else {
            Duration::from_micros(10)
        };

        match test_not_stuck(Flex::new(pin.reborrow()), delay).await {
            Ok(_) => info!("✔ {} is not stuck", NamedPin(&**pin)),
            Err(e) => {
                errors = true;
                error!("✘ {} is stuck! {}", NamedPin(&**pin), e);
            }
        }
    }

    let non_alt_pins = &mut pins[..22];

    let [from, .., to] = non_alt_pins else {
        defmt::unreachable!()
    };

    trace!(
        "Checking if pins {}  to {} are not shorted",
        NamedPin(&**from),
        NamedPin(&**to)
    );

    let mut flex: heapless::Vec<_, 22> = non_alt_pins
        .iter_mut()
        .map(|p| Flex::new(p.reborrow()))
        .collect();

    match test_not_interconnected(flex.as_mut_slice(), Duration::from_millis(10)).await {
        Ok(_) => info!("✔ No shorts"),
        Err(_) => {
            errors = true;
            error!("✘ Found shorts")
        }
    }

    for (i, switch) in [&mut shield.usb_power0, &mut shield.usb_power1]
        .into_iter()
        .enumerate()
    {
        match test_power_switch(switch).await {
            Ok(_) => info!("✔ USB Port {} works", i),
            Err(_) => {
                errors = true;
                error!("✘ USB Port {} does not work", i)
            }
        }
    }

    if errors { Err(()) } else { Ok(()) }
}
