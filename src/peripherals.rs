use std::mem;
use esp_idf_svc::hal::timer;

const PIN_COUNT: usize = 24;
const TIMERS_COUNT: usize = 2;
//const PINS: [u8; 24] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23];
const DIGITAL_PINS_BOUNDS: (usize, usize) = (0,23);
const ANALOG_PINS_BOUNDS: (usize, usize) = (0, 6);
const TIMER_BOUND: (usize, usize) = (0,1);

pub enum Peripheral{
    Pin(u8),
    Timer(u8),
    Adc,
    None
}

pub enum Pin{
    gpio(Gpio0)
}

impl Default for Peripheral {
    fn default() -> Self {
        Peripheral::None
    }
}

impl Peripheral {
    fn take(&mut self) -> Peripheral {
        mem::take(self)
    }
}


pub struct Peripherals {
    pins: [Peripheral;PIN_COUNT],
    timers: [Peripheral; TIMERS_COUNT],
    adc: Peripheral
}

impl Peripherals {
    pub fn new() -> Peripherals {
        let pins: [Peripheral; PIN_COUNT] = [Peripheral::Pin(0), Peripheral::Pin(1), Peripheral::Pin(2), Peripheral::Pin(3), Peripheral::Pin(4), Peripheral::Pin(5), Peripheral::Pin(6), Peripheral::Pin(7), Peripheral::Pin(8), Peripheral::Pin(9), Peripheral::Pin(10), Peripheral::Pin(11), Peripheral::Pin(12), Peripheral::Pin(13), Peripheral::None, Peripheral::Pin(15), Peripheral::Pin(16), Peripheral::Pin(17), Peripheral::Pin(18), Peripheral::Pin(19), Peripheral::Pin(20), Peripheral::Pin(21), Peripheral::Pin(22), Peripheral::Pin(23)];
        let timers: [Peripheral; TIMERS_COUNT] = [Peripheral::Timer(0), Peripheral::Timer(1)];
        let adc: Peripheral = Peripheral::Adc;
        Peripherals {
            pins,
            timers,
            adc
        }
    }

    pub fn get_digital_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, DIGITAL_PINS_BOUNDS)
    }

    pub fn get_analog_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, ANALOG_PINS_BOUNDS)
    }

    fn get_pin_on_bound(&mut self, pin_num: usize, bound: (usize,usize)) -> Peripheral {
        if pin_num >= bound.0 && pin_num <= bound.1 {
            return self.pins[pin_num].take()
        }
        Peripheral::None
    }

    pub fn get_timer(&mut self, timer_num: usize) -> Peripheral {
        if timer_num >= TIMER_BOUND.0 && timer_num <= TIMER_BOUND.1 {
            return self.timers[timer_num].take()
        }
        Peripheral::None
    }

    pub fn get_adc(&mut self) -> Peripheral {
        self.adc.take()
    }
}