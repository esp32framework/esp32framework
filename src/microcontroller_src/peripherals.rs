use esp32_nimble::BLEDevice;
use esp_idf_svc::hal::{gpio::*, i2c::I2C0, modem};
use std::mem;

const PIN_COUNT: usize = 24;
const TIMERS_COUNT: usize = 2;
const PWM_COUNT: usize = 4;
const DIGITAL_PINS_BOUNDS: (usize, usize) = (0, 23);
const PWM_PIN_BOUNDS: (usize, usize) = (0, 23);
const ANALOG_PINS_BOUNDS: (usize, usize) = (0, 6);
const TIMER_BOUND: (usize, usize) = (0, 1);
const UART_COUNT: usize = 2;
const UART_BOUNDS: (usize, usize) = (0, 1);

/// Error types related to microcontroller peripheral operations.
#[derive(Debug)]
pub enum PeripheralError {
    AlreadyTaken,
    NotABleDevicePeripheral,
    NotAnI2CPeripheral,
    NotAPin,
    NotAPwmTimer,
    NotAPwmChannel,
    NotATimerGroup,
}

/// Represents the esp32 Peripheral allowing to instanciate diferent Peripheral Types
#[derive(Default)]
pub enum Peripheral {
    Pin(u8),
    Timer(u8),
    PWMChannel(u8),
    PWMTimer(u8),
    Adc,
    I2C,
    Uart(u8),
    BleDevice,
    Modem,
    #[default]
    None,
}

impl Peripheral {
    /// Takes the Peripheral instance and changes to a Peripheral::None instance.
    ///
    /// # Returns
    ///
    /// A `Peripheral` instance. It it was already a `Peripheral::None` it will keep returning it.
    fn take(&mut self) -> Peripheral {
        mem::take(self)
    }

    /// Transforms the Peripheral instance into a AnyIOPin
    ///
    /// If the Peripheral is a Pin returns the corresponding AnyIoPin.
    /// If not it returns PeripheralError::NotAPin
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnyIOPin` instance, or an `PeripheralError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `PeripheralError::NotAPin`: If the pin number is invalid or the Peripheral is not of type Pin
    pub fn into_any_io_pin(self) -> Result<AnyIOPin, PeripheralError> {
        let pin = match self {
            Peripheral::Pin(pin_num) => match pin_num {
                0 => unsafe { Gpio0::new().downgrade() },
                1 => unsafe { Gpio1::new().downgrade() },
                2 => unsafe { Gpio2::new().downgrade() },
                3 => unsafe { Gpio3::new().downgrade() },
                4 => unsafe { Gpio4::new().downgrade() },
                5 => unsafe { Gpio5::new().downgrade() },
                6 => unsafe { Gpio6::new().downgrade() },
                7 => unsafe { Gpio7::new().downgrade() },
                8 => unsafe { Gpio8::new().downgrade() },
                9 => unsafe { Gpio9::new().downgrade() },
                10 => unsafe { Gpio10::new().downgrade() },
                11 => unsafe { Gpio11::new().downgrade() },
                12 => unsafe { Gpio12::new().downgrade() },
                13 => unsafe { Gpio13::new().downgrade() },
                15 => unsafe { Gpio15::new().downgrade() },
                16 => unsafe { Gpio16::new().downgrade() },
                17 => unsafe { Gpio17::new().downgrade() },
                18 => unsafe { Gpio18::new().downgrade() },
                19 => unsafe { Gpio19::new().downgrade() },
                20 => unsafe { Gpio20::new().downgrade() },
                21 => unsafe { Gpio21::new().downgrade() },
                22 => unsafe { Gpio22::new().downgrade() },
                23 => unsafe { Gpio23::new().downgrade() },
                _ => return Err(PeripheralError::NotAPin),
            },
            Peripheral::None => return Err(PeripheralError::AlreadyTaken),
            _ => return Err(PeripheralError::NotAPin),
        };
        Ok(pin)
    }

    /// Transforms the Peripheral instance into a I2C0
    ///
    /// If the Peripheral is a I2C returns the corresponding I2C0.
    /// If its a None it returns PeripheralError::AlreadyTaken
    /// Otherwise it returns PeripheralError::NotAnI2CPeripheral
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `I2C0` instance, or an `PeripheralError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `PeripheralError::NotAPin`: If the Peripheral is not of type I2C
    /// - `PeripheralError::NotAnI2CPeripheral`: Peripheral can not be transform into a I2C.
    pub fn into_i2c0(self) -> Result<I2C0, PeripheralError> {
        match self {
            Peripheral::I2C => Ok(unsafe { I2C0::new() }),
            Peripheral::None => Err(PeripheralError::AlreadyTaken),
            _ => Err(PeripheralError::NotAnI2CPeripheral),
        }
    }

    /// Transforms the Peripheral instance into a BleDevice.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `BLEDevice` instance, or an `PeripheralError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `PeripheralError::AlreadyTaken`: If the BleDevice was already taken.
    /// - `PeripheralError::NotABleDevicePeripheral`: Peripheral can not be transform into a BleDevice.
    pub fn into_ble_device(self) -> Result<&'static mut BLEDevice, PeripheralError> {
        match self {
            Peripheral::BleDevice => Ok(BLEDevice::take()),
            Peripheral::None => Err(PeripheralError::AlreadyTaken),
            _ => Err(PeripheralError::NotABleDevicePeripheral),
        }
    }

    /// Transforms the Peripheral instance into a Modem.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Modem` instance, or an `PeripheralError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `PeripheralError::AlreadyTaken`: If the Modem was already taken.
    /// - `PeripheralError::NotABleDevicePeripheral`: Peripheral can not be transform into a Modem.
    pub fn into_modem(self) -> Result<modem::Modem, PeripheralError> {
        match self {
            Peripheral::BleDevice => Ok(unsafe { modem::Modem::new() }),
            Peripheral::None => Err(PeripheralError::AlreadyTaken),
            _ => Err(PeripheralError::NotABleDevicePeripheral),
        }
    }
}

/// Represents the available peripherals in the esp32C6 and provides a way to get each particular
/// peripheral. Subsequent gets of the same peripheral will return Peripheral::None. On some cases
/// the same peripheral can be obatained by different getters, but only the first one will return the
/// pin.
pub struct Peripherals {
    pins: [Peripheral; PIN_COUNT],
    timers: [Peripheral; TIMERS_COUNT],
    pwm_channels: [Peripheral; PWM_COUNT],
    pwm_timers: [Peripheral; PWM_COUNT],
    adc: Peripheral,
    i2c: Peripheral,
    uart: [Peripheral; UART_COUNT],
    ble_device: Peripheral,
    modem: Peripheral,
}

impl Peripherals {
    /// Creates a new Peripherals instance.
    ///
    /// # Returns
    ///
    /// The new Peripherals instance with every Peripheral available
    pub(crate) fn new() -> Peripherals {
        let pins = Self::new_pins();
        let timers: [Peripheral; TIMERS_COUNT] = [Peripheral::Timer(0), Peripheral::Timer(1)];
        let pwm_channels = Self::new_pwm_channels();
        let pwm_timers = Self::new_pwm_timers();
        let adc: Peripheral = Peripheral::Adc;
        let i2c: Peripheral = Peripheral::I2C;
        let uart: [Peripheral; UART_COUNT] = [Peripheral::Uart(0), Peripheral::Uart(1)];
        let ble_device = Peripheral::BleDevice;
        let modem = Peripheral::Modem;
        Peripherals {
            pins,
            timers,
            pwm_channels,
            pwm_timers,
            adc,
            i2c,
            uart,
            ble_device,
            modem,
        }
    }

    /// Creates the peripherls corresponding to the pwm timers
    fn new_pwm_timers() -> [Peripheral; PWM_COUNT] {
        [
            Peripheral::PWMTimer(0),
            Peripheral::PWMTimer(1),
            Peripheral::PWMTimer(2),
            Peripheral::PWMTimer(3),
        ]
    }

    /// Creates the peripherls corresponding to the pwm channels
    fn new_pwm_channels() -> [Peripheral; PWM_COUNT] {
        [
            Peripheral::PWMChannel(0),
            Peripheral::PWMChannel(1),
            Peripheral::PWMChannel(2),
            Peripheral::PWMChannel(3),
        ]
    }

    /// Creates the peripherls corresponding to the pins
    fn new_pins() -> [Peripheral; PIN_COUNT] {
        [
            Peripheral::Pin(0),
            Peripheral::Pin(1),
            Peripheral::Pin(2),
            Peripheral::Pin(3),
            Peripheral::Pin(4),
            Peripheral::Pin(5),
            Peripheral::Pin(6),
            Peripheral::Pin(7),
            Peripheral::Pin(8),
            Peripheral::Pin(9),
            Peripheral::Pin(10),
            Peripheral::Pin(11),
            Peripheral::Pin(12),
            Peripheral::Pin(13),
            Peripheral::None,
            Peripheral::Pin(15),
            Peripheral::Pin(16),
            Peripheral::Pin(17),
            Peripheral::Pin(18),
            Peripheral::Pin(19),
            Peripheral::Pin(20),
            Peripheral::Pin(21),
            Peripheral::Pin(22),
            Peripheral::Pin(23),
        ]
    }

    /// Gets the desired pin Peripheral
    ///
    /// # Arguments
    ///
    /// - `pin_num`: An usize representing the desired digital pin number. Accepted values go from 0 to 23 inclusive.
    ///
    /// # Returns
    ///
    /// A `Peripheral` that may be a `Peripheral::Pin` instance, or a `Peripheral::None` instance if the
    /// desired pin number is not available for digital pins or bacause the same pin number was already taken before.
    pub fn get_digital_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, DIGITAL_PINS_BOUNDS)
    }

    /// Gets the desired pin Peripheral
    ///
    /// # Arguments
    ///
    /// - `pin_num`: An usize representing the desired analog pin number. Accepted values go from 0 to 6 inclusive.
    ///
    /// # Returns
    ///
    /// A `Peripheral` that may be a `Peripheral::Pin` instance, or a `Peripheral::None` instance if the
    /// desired pin number is not available for analog pins or bacause the same pin number was already taken before.
    pub fn get_analog_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, ANALOG_PINS_BOUNDS)
    }

    /// Gets the desired pin Peripheral
    ///
    /// # Arguments
    ///
    /// - `pin_num`: An usize representing the desired digital pin number, since PWM uses a digital pin. Accepted values go from 0 to 23 inclusive.
    ///
    /// # Returns
    ///
    /// A `Peripheral` that may be a `Peripheral::Pin` instance, or a `Peripheral::None` instance if the
    /// desired pin number is not available for digital pins or bacause the same pin number was already taken before.
    pub fn get_pwm_pin(&mut self, pin_num: usize) -> Peripheral {
        self.get_pin_on_bound(pin_num, PWM_PIN_BOUNDS)
    }

    /// Checks if the desired pin number is between accepted range for a specific peripheral purpose
    ///
    /// # Arguments
    ///
    ///  - `pin_num`: An usize representing the usize to check if it is between bounds
    ///  - `bound`: A tuple with 2 usize. It represents the limits of the accepted range. The format is (min_bound, max_bound)
    ///
    /// # Returns
    ///
    /// A `Peripheral` that may be a `Peripheral::Pin` instance, or a `Peripheral::None` instance if the
    /// desired pin number is not between bounds or bacause the same pin number was already taken before.
    fn get_pin_on_bound(&mut self, pin_num: usize, bound: (usize, usize)) -> Peripheral {
        if pin_num >= bound.0 && pin_num <= bound.1 {
            return self.pins[pin_num].take();
        }
        Peripheral::None
    }

    /// Gets the desired timer Peripheral
    ///
    /// # Arguments
    ///
    /// - `timer_num`: An usize representing the desired timer number. Accepted values are 0 or 1.
    ///
    /// # Returns
    ///
    /// A `Peripheral` that may be a `Peripheral::Timer` instance, or a `Peripheral::None` instance if the
    /// desired timer number is not available for timers or bacause the same timer number was already taken before.
    pub fn get_timer(&mut self, timer_num: usize) -> Peripheral {
        if timer_num >= TIMER_BOUND.0 && timer_num <= TIMER_BOUND.1 {
            return self.timers[timer_num].take();
        }
        Peripheral::None
    }

    /// Gets the only ADC peripheral available
    ///
    /// # Returns
    ///
    /// A `Peripheral::Adc` if it was not taken before, otherwise a `Peripheral::None`
    pub fn get_adc(&mut self) -> Peripheral {
        self.adc.take()
    }

    /// Gets the next PWM Channel peripheral and PWM Timer peripheral available
    ///
    /// # Returns
    ///
    /// A tuple containing the `Peripheral::PWMChannel` and the `Peripheral::PWMTimer` if both of them are still available,
    ///  otherwise a tuple containing two `Peripheral::None`.
    pub fn get_next_pwm(&mut self) -> (Peripheral, Peripheral) {
        for (channel, timer) in self.pwm_channels.iter_mut().zip(self.pwm_timers.iter_mut()) {
            if let Peripheral::None = channel {
                continue;
            }

            return (channel.take(), timer.take());
        }
        (Peripheral::None, Peripheral::None)
    }

    /// Gets the only I2C peripheral available
    ///
    /// # Returns
    ///
    /// A `Peripheral::I2C` if it was not taken before, otherwise a `Peripheral::None`
    pub fn get_i2c(&mut self) -> Peripheral {
        self.i2c.take()
    }

    /// Gets the desired uart Peripheral
    ///
    /// # Arguments
    ///
    /// - `uart_num`: An usize representing the desired uart number. Accepted values are 0 or 1.
    ///
    /// # Returns
    ///
    /// A `Peripheral` that may be a `Peripheral::Uart` instance, or a `Peripheral::None` instance if the
    /// desired uart number is not available for uart drivers or bacause the same uart driver number was already taken before.
    pub fn get_uart(&mut self, uart_num: usize) -> Peripheral {
        if uart_num >= UART_BOUNDS.0 && uart_num <= UART_BOUNDS.1 {
            return self.uart[uart_num].take();
        }
        Peripheral::None
    }

    /// Gets the only BleDevice peripheral available
    ///
    /// # Returns
    ///
    /// A `Peripheral::BleDevice` if it was not taken before, otherwise a `Peripheral::None
    pub fn get_ble_peripheral(&mut self) -> Peripheral {
        self.ble_device.take()
    }

    /// Gets the only Modem peripheral available
    ///
    /// # Returns
    ///
    /// A `Peripheral::Modem` if it was not taken before, otherwise a `Peripheral::None
    pub fn get_wifi_peripheral(&mut self) -> Peripheral {
        self.modem.take()
    }
}
