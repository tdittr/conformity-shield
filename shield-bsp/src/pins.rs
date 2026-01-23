use embassy_rp::{
    Peri,
    peripherals::{
        PIN_0, PIN_1, PIN_2, PIN_3, PIN_4, PIN_5, PIN_6, PIN_7, PIN_8, PIN_9, PIN_10, PIN_11,
        PIN_12, PIN_13, PIN_14, PIN_15, PIN_16, PIN_17, PIN_18, PIN_19, PIN_20, PIN_21, PIN_22,
        PIN_23, PIN_24, PIN_25, PIN_26, PIN_27, PIN_28, PIN_29, PIN_30, PIN_31, PIN_32, PIN_33,
        PIN_34, PIN_35, PIN_36, PIN_37, PIN_38, PIN_39, PIN_40, PIN_41, PIN_42, PIN_43, PIN_44,
        PIN_45, PIN_46, PIN_47,
    },
};

pub struct DutPins {
    pub d00: Peri<'static, PIN_0>,
    pub d01: Peri<'static, PIN_1>,
    pub d02: Peri<'static, PIN_2>,
    pub d03: Peri<'static, PIN_3>,
    pub d04: Peri<'static, PIN_4>,
    pub d05: Peri<'static, PIN_5>,
    pub d06: Peri<'static, PIN_6>,
    pub d07: Peri<'static, PIN_7>,
    pub d08: Peri<'static, PIN_8>,
    pub d09: Peri<'static, PIN_9>,
    pub d10: Peri<'static, PIN_10>,
    pub d11: Peri<'static, PIN_11>,
    pub d12: Peri<'static, PIN_12>,
    pub d13: Peri<'static, PIN_13>,
    pub d14: Peri<'static, PIN_14>,
    pub d15: Peri<'static, PIN_15>,

    pub d10_alt: Peri<'static, PIN_25>,
    pub d11_alt: Peri<'static, PIN_24>,
    pub d12_alt: Peri<'static, PIN_27>,
    pub d13_alt: Peri<'static, PIN_26>,

    pub a0: Peri<'static, PIN_40>,
    pub a1: Peri<'static, PIN_41>,
    pub a2: Peri<'static, PIN_42>,
    pub a3: Peri<'static, PIN_43>,
    pub a4: Peri<'static, PIN_44>,
    pub a5: Peri<'static, PIN_45>,
    pub a6: Peri<'static, PIN_46>,

    pub pwm0: Peri<'static, PIN_39>,
    pub pwm1: Peri<'static, PIN_38>,
    pub pwm2: Peri<'static, PIN_37>,
    pub pwm3: Peri<'static, PIN_36>,
    pub pwm4: Peri<'static, PIN_35>,
    pub pwm5: Peri<'static, PIN_34>,
    pub pwm6: Peri<'static, PIN_33>,

    pub reset: Peri<'static, PIN_32>,
}

pub struct ShieldPins {
    pub usb_host_en: Peri<'static, PIN_16>,
    pub usb_host_sel: Peri<'static, PIN_17>,
    pub usb_host_dp: Peri<'static, PIN_18>,
    pub usb_host_dn: Peri<'static, PIN_19>,

    pub usb0_en: Peri<'static, PIN_20>,
    pub usb0_flag: Peri<'static, PIN_21>,
    pub usb1_en: Peri<'static, PIN_22>,
    pub usb1_flag: Peri<'static, PIN_23>,

    pub todi: Peri<'static, PIN_28>,
    pub tido: Peri<'static, PIN_29>,

    pub sda: Peri<'static, PIN_30>,
    pub scl: Peri<'static, PIN_31>,

    pub led: Peri<'static, PIN_47>,
}
