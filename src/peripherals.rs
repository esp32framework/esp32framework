use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals;

pub struct Peripherals {
    gpio0: Option<Gpio0>,
    gpio1: Option<Gpio1>,
    gpio2: Option<Gpio2>,
    gpio3: Option<Gpio3>,
    gpio4: Option<Gpio4>,
    gpio5: Option<Gpio5>,
    gpio6: Option<Gpio6>,
    gpio7: Option<Gpio7>,
    gpio8: Option<Gpio8>,
    gpio9: Option<Gpio9>,
    gpio10: Option<Gpio10>,
    gpio11: Option<Gpio11>,
    gpio12: Option<Gpio12>,
    gpio13: Option<Gpio13>,
    gpio14: Option<Gpio14>,
    gpio15: Option<Gpio15>,
    gpio16: Option<Gpio16>,
    gpio17: Option<Gpio17>,
    gpio18: Option<Gpio18>,
    gpio19: Option<Gpio19>,
    gpio20: Option<Gpio20>,
    gpio21: Option<Gpio21>,
    gpio22: Option<Gpio22>,
    gpio23: Option<Gpio23>,
    gpio24: Option<Gpio24>,
    gpio25: Option<Gpio25>,
    gpio26: Option<Gpio26>,
    gpio27: Option<Gpio27>,
    gpio28: Option<Gpio28>,
    gpio29: Option<Gpio29>,
    gpio30: Option<Gpio30>,
}

impl Peripherals {
    pub fn new()->Self{
        let p = peripherals::Peripherals::take().expect("Cannotn acces pins");
        Peripherals{
            gpio0: Some(p.pins.gpio0),
            gpio1: Some(p.pins.gpio1),
            gpio2: Some(p.pins.gpio2),
            gpio3: Some(p.pins.gpio3),
            gpio4: Some(p.pins.gpio4),
            gpio5: Some(p.pins.gpio5),
            gpio6: Some(p.pins.gpio6),
            gpio7: Some(p.pins.gpio7),
            gpio8: Some(p.pins.gpio8),
            gpio9: Some(p.pins.gpio9),
            gpio10: Some(p.pins.gpio10),
            gpio11: Some(p.pins.gpio11),
            gpio12: Some(p.pins.gpio12),
            gpio13: Some(p.pins.gpio13),
            gpio14: Some(p.pins.gpio14),
            gpio15: Some(p.pins.gpio15),
            gpio16: Some(p.pins.gpio16),
            gpio17: Some(p.pins.gpio17),
            gpio18: Some(p.pins.gpio18),
            gpio19: Some(p.pins.gpio19),
            gpio20: Some(p.pins.gpio20),
            gpio21: Some(p.pins.gpio21),
            gpio22: Some(p.pins.gpio22),
            gpio23: Some(p.pins.gpio23),
            gpio24: Some(p.pins.gpio24),
            gpio25: Some(p.pins.gpio25),
            gpio26: Some(p.pins.gpio26),
            gpio27: Some(p.pins.gpio27),
            gpio28: Some(p.pins.gpio28),
            gpio29: Some(p.pins.gpio29),
            gpio30: Some(p.pins.gpio30)
            }
        }
}   