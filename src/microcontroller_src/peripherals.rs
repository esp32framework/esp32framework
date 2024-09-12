use std::mem;
use esp_idf_svc::hal::gpio::*;

const PIN_COUNT: usize = 24;
const TIMERS_COUNT: usize = 2;
const PWM_COUNT: usize = 4;
const DIGITAL_PINS_BOUNDS: (usize, usize) = (0,23);
const PWM_PIN_BOUNDS: (usize, usize) = (0,23);
const ANALOG_PINS_BOUNDS: (usize, usize) = (0, 6);
const TIMER_BOUND: (usize, usize) = (0,1);
const UART_COUNT: usize = 2;
const UART_BOUNDS: (usize, usize) = (0, 1);


#[derive(Debug)]
pub enum PeripheralError {
    NotAPin
}

/// Represents the esp32 Peripheral allowing to instanciate diferent Peripheral Types 
#[derive(Default)]
pub enum Peripheral{
    Pin(u8),
    Timer(u8),
    PWMChannel(u8),
    PWMTimer(u8),
    Adc,
    I2C,
    Uart(u8),
    BleDevice,
    #[default]
    None
}

impl Peripheral {
    fn take(&mut self) -> Peripheral {
        mem::take(self)
    }

    /// If the Peripheral is a Pin returns the corresponding AnyIoPin. 
    /// If not it returns PeripheralError::NotAPin
    pub fn into_any_io_pin(self) -> Result<AnyIOPin, PeripheralError> {
        let pin = match self {
            Peripheral::Pin(pin_num) => match pin_num{
                0 => unsafe {Gpio0::new().downgrade()},
                1 => unsafe {Gpio1::new().downgrade()},
                2 => unsafe {Gpio2::new().downgrade()},
                3 => unsafe {Gpio3::new().downgrade()},
                4 => unsafe {Gpio4::new().downgrade()},
                5 => unsafe {Gpio5::new().downgrade()},
                6 => unsafe {Gpio6::new().downgrade()},
                7 => unsafe {Gpio7::new().downgrade()},
                8 => unsafe {Gpio8::new().downgrade()},
                9 => unsafe {Gpio9::new().downgrade()},
                10 => unsafe {Gpio10::new().downgrade()},
                11 => unsafe {Gpio11::new().downgrade()},
                12 => unsafe {Gpio12::new().downgrade()},
                13 => unsafe {Gpio13::new().downgrade()},
                15 => unsafe {Gpio15::new().downgrade()},
                16 => unsafe {Gpio16::new().downgrade()},
                17 => unsafe {Gpio17::new().downgrade()},
                18 => unsafe {Gpio18::new().downgrade()},
                19 => unsafe {Gpio19::new().downgrade()},
                20 => unsafe {Gpio20::new().downgrade()},
                21 => unsafe {Gpio21::new().downgrade()},
                22 => unsafe {Gpio22::new().downgrade()},
                23 => unsafe {Gpio23::new().downgrade()},
                _ => return Err(PeripheralError::NotAPin)
            },
            _ => return Err(PeripheralError::NotAPin),
        };
        Ok(pin)
    }
}

/// Represents the available peripherals in the esp32C6 and provides a way to get each particular
/// peripheral. Subsequent gets of the same peripheral will return Peripheral::None. On some cases
/// the same peripheral can be obatained by different getters, but only the first one will return the
/// pin. 
pub struct Peripherals {
    pins: [Peripheral;PIN_COUNT],
    timers: [Peripheral; TIMERS_COUNT],
    pwm_channels: [Peripheral; PWM_COUNT],
    pwm_timers: [Peripheral; PWM_COUNT],
    adc: Peripheral,
    i2c: Peripheral,
    uart: [Peripheral; UART_COUNT],
    ble_device: Peripheral
}

impl Peripherals {
    pub fn new() -> Peripherals {
        let pins: [Peripheral; PIN_COUNT] = [Peripheral::Pin(0), Peripheral::Pin(1), Peripheral::Pin(2), Peripheral::Pin(3), Peripheral::Pin(4), Peripheral::Pin(5), Peripheral::Pin(6), Peripheral::Pin(7), Peripheral::Pin(8), Peripheral::Pin(9), Peripheral::Pin(10), Peripheral::Pin(11), Peripheral::Pin(12), Peripheral::Pin(13), Peripheral::None, Peripheral::Pin(15), Peripheral::Pin(16), Peripheral::Pin(17), Peripheral::Pin(18), Peripheral::Pin(19), Peripheral::Pin(20), Peripheral::Pin(21), Peripheral::Pin(22), Peripheral::Pin(23)];
        let timers: [Peripheral; TIMERS_COUNT] = [Peripheral::Timer(0), Peripheral::Timer(1)];
        let pwm_channels: [Peripheral; PWM_COUNT] = [Peripheral::PWMChannel(0),Peripheral::PWMChannel(1),Peripheral::PWMChannel(2),Peripheral::PWMChannel(3)];
        let pwm_timers: [Peripheral; PWM_COUNT] = [Peripheral::PWMTimer(0), Peripheral::PWMTimer(1), Peripheral::PWMTimer(2), Peripheral::PWMTimer(3)];
        let adc: Peripheral = Peripheral::Adc;
        let i2c: Peripheral = Peripheral::I2C;
        let uart: [Peripheral; UART_COUNT] = [Peripheral::Uart(0), Peripheral::Uart(1)];
        let ble_device = Peripheral::BleDevice;
        Peripherals {
            pins,
            timers,
            pwm_channels,
            pwm_timers,
            adc,
            i2c,
            uart,
            ble_device
        }
    }

    pub fn get_digital_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, DIGITAL_PINS_BOUNDS)
    }

    pub fn get_analog_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, ANALOG_PINS_BOUNDS)
    }
    
    pub fn get_pwm_pin(&mut self, pin_num: usize) -> Peripheral{
        self.get_pin_on_bound(pin_num, PWM_PIN_BOUNDS)
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

    pub fn get_next_pwm(&mut self) -> (Peripheral, Peripheral) {
        
        for (channel, timer) in self.pwm_channels.iter_mut().zip(self.pwm_timers.iter_mut()) {
            if let Peripheral::None = channel{
                continue
            }

            return (channel.take(), timer.take())
        }
        (Peripheral::None, Peripheral::None)
    }

    pub fn get_i2c(&mut self) -> Peripheral {
        self.i2c.take()
    }

    pub fn get_uart(&mut self, uart_num: usize) -> Peripheral {
        if uart_num >= UART_BOUNDS.0 && uart_num <= UART_BOUNDS.1 {
            return self.uart[uart_num].take()
        }
        Peripheral::None
    }

    pub fn get_ble_device(&mut self)-> Peripheral{
        self.ble_device.take()
    }
}
