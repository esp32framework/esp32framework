use crate::{
    ble::{
        utils::{Security, Service},
        BleBeacon, BleClient, BleError, BleServer,
    },
    gpio::{analog::*, digital::*},
    microcontroller_src::{interrupt_driver::InterruptDriver, peripherals::*},
    serial::{i2c::*, uart::*},
    timer_driver::TimerDriverError,
    utils::{
        auxiliary::{SharableRef, SharableRefExt},
        esp32_framework_error::{AdcDriverError, Esp32FrameworkError},
        notification::{Notification, Notifier},
        timer_driver::TimerDriver,
    },
    wifi::{WifiDriver, WifiError},
};
use attenuation::adc_atten_t;
use esp32_nimble::{enums::AuthReq, BLEDevice};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{adc::*, delay::FreeRtos, task::block_on},
};
use futures::future::{join, Future};
use oneshot::AdcDriver;
use std::{
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
};

const TIMER_GROUPS: usize = 2;

pub(crate) type SharableAdcDriver<'a> = Rc<AdcDriver<'a, ADC1>>;
static TAKEN: AtomicBool = AtomicBool::new(false);

/// Primary abstraction for interacting with the microcontroller, providing access to peripherals and drivers
/// required for configuring pins and other functionalities.
///
/// - `peripherals`: An instance of `Peripherals`, representing the various hardware peripherals available on the microcontroller.
/// - `timer_drivers`: A vector of `TimerDriver` instances, each associated with a timer peripheral for time-based operations.
/// - `interrupt_drivers`: A vector of boxed `InterruptDriver` trait objects, representing the drivers responsible for handling hardware interrupts.
/// - `adc_driver`: An optional shared instance of `SharableAdcDriver`, providing access to the ADC (Analog-to-Digital Converter) for analog input processing.
/// - `notification`: An instance of `Notification`, used for managing notifications or signaling events within the microcontroller's operation.
pub struct Microcontroller<'a> {
    peripherals: Peripherals,
    timer_drivers: Vec<TimerDriver<'a>>,
    interrupt_drivers: Vec<Box<dyn InterruptDriver<'a> + 'a>>,
    adc_driver: Option<SharableAdcDriver<'a>>,
    notification: Notification,
    event_loop: EspSystemEventLoop,
}

impl<'a> Microcontroller<'a> {
    /// Creates a new Microcontroller instance.
    ///
    /// # Returns
    ///
    /// The new Microcontroller
    ///
    /// # Panics
    ///
    /// Panics if the microcontroller was already initialized. In this case, the microcontroller
    /// remains in an invalid state.
    pub fn take() -> Self {
        Microcontroller::assert_uniqueness().unwrap();

        esp_idf_svc::sys::link_patches();
        let mut peripherals = Peripherals::new();
        let notification = Notification::new();
        let timer_drivers =
            Microcontroller::initialize_timer_drivers(&mut peripherals, &notification).unwrap();

        Microcontroller {
            peripherals,
            timer_drivers,
            interrupt_drivers: Vec::new(),
            adc_driver: None,
            notification,
            event_loop: EspSystemEventLoop::take().expect("Error creating microcontroller"),
        }
    }

    /// Asserts whether another instance of microcontroller exists.
    ///
    /// #Returns
    ///
    /// A Ok() if there is no other instance of the microcontroller or Err(`Esp32FrameworkError`) if one already exists.
    ///
    /// # Errors
    ///
    /// - `Esp32FrameworkError::CantHaveMoreThanOneMicrocontroller`: If an instance of microcontroller already exists.
    fn assert_uniqueness() -> Result<(), Esp32FrameworkError> {
        if TAKEN.load(Ordering::SeqCst) {
            return Err(Esp32FrameworkError::CantHaveMoreThanOneMicrocontroller);
        }
        TAKEN.store(true, Ordering::Relaxed);
        Ok(())
    }

    /// Creates the timer_drivers for all timer groups
    ///
    /// #Returns
    ///
    /// A `Result` containing the timer drivers or an instance of a `TimerDriverError` if the creation fails.
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::InvalidTimer`: If the peripheral used is not a Peripheral::Timer  
    fn initialize_timer_drivers(
        peripherals: &mut Peripherals,
        notification: &Notification,
    ) -> Result<Vec<TimerDriver<'a>>, TimerDriverError> {
        let mut timer_drivers = Vec::new();
        for i in 0..TIMER_GROUPS {
            let timer = peripherals.get_timer(i);
            timer_drivers.push(TimerDriver::new(timer, notification.notifier())?);
        }
        Ok(timer_drivers)
    }

    /// Retrieves a TimerDriver instance.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `TimerDriver` instance or a `TimerDriverError` if the creation fails.
    ///
    /// A `TimerDriver` instance can be used to manage timers in the microcontroller.
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::TooManyChildren`: If too many timer drivers have been created
    pub fn get_timer_driver(&mut self) -> Result<TimerDriver<'a>, TimerDriverError> {
        let mut timer_driver = self.timer_drivers.swap_remove(0);
        let timer_driver_copy = timer_driver.create_child_copy()?;
        self.timer_drivers.push(timer_driver);
        Ok(timer_driver_copy)
    }

    /// Creates a DigitalIn on the ESP pin with number 'pin_num' to read digital inputs.
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as a digital input.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `DigitalIn` instance that can be used to read digital inputs from the specified pin, or
    /// a `DigitalInError` if the setting fails
    ///
    /// # Errors
    ///
    /// - `DigitalInError::TimerDriver`: This error is returned if an issue occurs while initializing the TimerDriver.
    /// - `DigitalInError::InvalidPeripheral`: If per parameter is not capable of transforming into an AnyIOPin
    /// - `DigitalInError::CannotSetPinAsInput`: If the per parameter is not capable of soportin input
    ///
    /// # Panics
    ///
    /// When setting Down the pull fails on the creation of the DigitalIn
    pub fn set_pin_as_digital_in(
        &mut self,
        pin_num: usize,
    ) -> Result<DigitalIn<'a>, DigitalInError> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let dgin = DigitalIn::new(
            self.get_timer_driver()?,
            pin_peripheral,
            Some(self.notification.notifier()),
        )?;
        self.interrupt_drivers.push(dgin.get_updater());
        Ok(dgin)
    }

    /// Creates a DigitalOut on the ESP pin with number 'pin_num' to write digital outputs.
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as a digital output.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `DigitalOut` instance that can be used to write digital outputs to the specified pin, or
    /// a `DigitalOutError` if the setting fails
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::TimerDriver`: This error is returned if an issue occurs while initializing the TimerDriver.
    /// - `DigitalOutError::InvalidPeripheral`: If the peripheral cannot be converted into an AnyIOPin.
    /// - `DigitalOutError::CannotSetPinAsOutput`: If the pin cannot be set as an output.
    pub fn set_pin_as_digital_out(
        &mut self,
        pin_num: usize,
    ) -> Result<DigitalOut<'a>, DigitalOutError> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let dgout = DigitalOut::new(self.get_timer_driver()?, pin_peripheral)?;
        self.interrupt_drivers.push(dgout.get_updater());
        Ok(dgout)
    }

    /// Starts an adc driver if no other was started before. Bitwidth is always set to 12, since
    /// the ESP32-C6 only allows that width
    ///
    /// # Returns
    ///
    /// A `Result` with an Empty Ok, or an `AdcDriverError` if the starting fails
    ///
    /// # Errors
    ///
    /// - `AdcDriverError::Code`: To represent other errors.
    fn start_adc_driver(&mut self) -> Result<(), AdcDriverError> {
        if self.adc_driver.is_none() {
            let adc1 = self.peripherals.get_adc().into_adc1()?;
            let driver = AdcDriver::new(adc1)?;
            self.adc_driver.replace(Rc::new(driver));
        };
        Ok(())
    }

    /// Creates an AnalogIn from a pin and a desired attenuation
    ///
    /// # Arguments
    ///
    /// - `pin_num`: An usize representing the desired pin number on the microcontroller
    /// - `pin_num`: The desired attenuation for the analog pin
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `AnalogIn` instance, or a `AnalogInError` if the creation fails
    ///
    /// # Errors
    ///
    /// - `AnalogInError::AdcDriverError`: If starting the ADC driver fails
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    fn set_pin_as_analog_in(
        &mut self,
        pin_num: usize,
        attenuation: adc_atten_t,
    ) -> Result<AnalogIn<'a>, AnalogInError> {
        self.start_adc_driver()?;
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        let adc_driver = self
            .adc_driver
            .clone()
            .ok_or(AnalogInError::AdcDriverError(AdcDriverError::AlreadyTaken))?;
        AnalogIn::new(pin_peripheral, adc_driver, attenuation)
    }

    /// Sets pin as analog input with attenuation set to 2.5dB
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::AdcDriverError`: If starting the ADC driver fails
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    pub fn set_pin_as_analog_in_low_atten(
        &mut self,
        pin_num: usize,
    ) -> Result<AnalogIn<'a>, AnalogInError> {
        self.set_pin_as_analog_in(pin_num, attenuation::DB_2_5)
    }

    /// Sets pin as analog input with attenuation set to 6dB
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::AdcDriverError`: If starting the ADC driver fails
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    pub fn set_pin_as_analog_in_medium_atten(
        &mut self,
        pin_num: usize,
    ) -> Result<AnalogIn<'a>, AnalogInError> {
        self.set_pin_as_analog_in(pin_num, attenuation::DB_6)
    }

    /// Sets pin as analog input with attenuation set to 11dB
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::AdcDriverError`: If starting the ADC driver fails
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    pub fn set_pin_as_analog_in_high_atten(
        &mut self,
        pin_num: usize,
    ) -> Result<AnalogIn<'a>, AnalogInError> {
        self.set_pin_as_analog_in(pin_num, attenuation::DB_11)
    }

    /// Sets pin as analog input with attenuation set to 0dB
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::AdcDriverError`: If starting the ADC driver fails
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    pub fn set_pin_as_analog_in_no_atten(
        &mut self,
        pin_num: usize,
    ) -> Result<AnalogIn<'a>, AnalogInError> {
        self.set_pin_as_analog_in(pin_num, attenuation::NONE)
    }

    /// Sets pin as analog output, with desired frequency and resolution
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    /// - `freq_hz`: An u32 representing the desired frequency in hertz
    /// - `resolution`: An u32 that represents the amount of bits in the desired output resolution. if 0 its set to 1 bit, >= 14
    ///     14 bits of resolution are set  
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `AnalogOut` instance, or a `AnalogOutError` if the setting fails
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriver`: This error is returned if an issue occurs while initializing the TimerDriver.
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    pub fn set_pin_as_analog_out(
        &mut self,
        pin_num: usize,
        freq_hz: u32,
        resolution: u32,
    ) -> Result<AnalogOut<'a>, AnalogOutError> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        let analog_out = AnalogOut::new(
            pwm_channel,
            pwm_timer,
            pin_peripheral,
            self.get_timer_driver()?,
            freq_hz,
            resolution,
        )?;
        self.interrupt_drivers.push(analog_out.get_updater());
        Ok(analog_out)
    }

    /// Sets pin as analog output, with a default frequency of 100 Hertz and resolution of 8 bits
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnalogOut` instance, or an `AnalogOutError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriver`: This error is returned if an issue occurs while initializing the TimerDriver.
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    pub fn set_pin_as_default_analog_out(
        &mut self,
        pin_num: usize,
    ) -> Result<AnalogOut<'a>, AnalogOutError> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        let analog_out = AnalogOut::default(
            pwm_channel,
            pwm_timer,
            pin_peripheral,
            self.get_timer_driver()?,
        )?;
        self.interrupt_drivers.push(analog_out.get_updater());
        Ok(analog_out)
    }

    /// Sets pin as analog input of PWM signals, with default signal frequency of 1000 Hertz
    ///
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnalogInPwm` instance, or an `AnalogInPwmError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInPwmError::TimerDriver`: This error is returned if an issue occurs while initializing the TimerDriver.
    pub fn set_pin_as_analog_in_pwm(
        &mut self,
        pin_num: usize,
    ) -> Result<AnalogInPwm<'a>, AnalogInPwmError> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let timer_driver = self.get_timer_driver()?;
        AnalogInPwm::default(timer_driver, pin_peripheral)
    }

    /// Configures the specified pins for I2C master mode.
    ///
    /// # Arguments
    ///
    /// - `sda_pin`: The pin number to be used as the SDA (Serial Data) line.
    /// - `scl_pin`: The pin number to be used as the SCL (Serial Clock) line.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `I2CMaster` instance, or an `I2CError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidPin`: If either the SDA or SCL pins cannot be converted to IO pins.
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::DriverError`: If there is an error initializing the driver.
    pub fn set_pins_for_i2c_master(
        &mut self,
        sda_pin: usize,
        scl_pin: usize,
    ) -> Result<I2CMaster<'a>, I2CError> {
        let sda_peripheral = self.peripherals.get_digital_pin(sda_pin);
        let scl_peripheral = self.peripherals.get_digital_pin(scl_pin);

        I2CMaster::new(sda_peripheral, scl_peripheral, self.peripherals.get_i2c())
    }

    /// Configures the specified pins for I2C slave mode and sets the slave address.
    ///
    /// # Arguments
    ///
    /// - `sda_pin`: The pin number to be used as the SDA (Serial Data) line.
    /// - `scl_pin`: The pin number to be used as the SCL (Serial Clock) line.
    /// - `slave_addr`: The address of the I2C slave device.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `I2CSlave` instance, or an `I2CError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidPin`: If either the SDA or SCL pins cannot be converted to IO pins.
    pub fn set_pins_for_i2c_slave(
        &mut self,
        sda_pin: usize,
        scl_pin: usize,
        slave_addr: u8,
    ) -> Result<I2CSlave<'a>, I2CError> {
        let sda_peripheral = self.peripherals.get_digital_pin(sda_pin);
        let scl_peripheral = self.peripherals.get_digital_pin(scl_pin);

        I2CSlave::new(
            sda_peripheral,
            scl_peripheral,
            self.peripherals.get_i2c(),
            slave_addr,
        )
    }

    /// Configures the specified pins for a default UART configuration.
    /// The default configuration is:
    /// - `baudrate`: 115_200 Hz.
    /// - `parity`: None parity bit.
    /// - `stopbit`: One stop bit.
    ///
    /// # Arguments
    ///
    /// - `tx_pin`: The pin number to be used for UART transmission (TX).
    /// - `rx_pin`: The pin number to be used for UART reception (RX).
    /// - `uart_num`: The UART number to be configured.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `UART` instance, or an `UARTError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::InvalidPin`: If either the TX or RX pins cannot be converted to IO pins.
    /// - `UARTError::InvalidUartNumber`: If an unsupported UART peripheral is selected.
    /// - `UARTError::DriverError`: If there is an error initializing the driver.
    pub fn set_pins_for_default_uart(
        &mut self,
        tx_pin: usize,
        rx_pin: usize,
        uart_num: usize,
    ) -> Result<UART<'a>, UARTError> {
        let tx_peripheral = self.peripherals.get_digital_pin(tx_pin);
        let rx_peripheral = self.peripherals.get_digital_pin(rx_pin);
        let uart_peripheral = self.peripherals.get_uart(uart_num);

        UART::default(tx_peripheral, rx_peripheral, uart_peripheral)
    }

    /// Configures the specified pins for a UART configuration with custom settings.
    ///
    /// # Arguments
    ///
    /// - `tx_pin`: The pin number to be used for UART transmission (TX).
    /// - `rx_pin`: The pin number to be used for UART reception (RX).
    /// - `uart_num`: The UART number to be configured.
    /// - `baudrate`: The baud rate for the UART communication.
    /// - `parity`: The parity setting for the UART.
    /// - `stopbit`: The stop bit configuration for the UART.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `UART` instance, or an `UARTError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::InvalidPin`: If either the TX or RX pins cannot be converted to IO pins.
    /// - `UARTError::InvalidUartNumber`: If an unsupported UART peripheral is selected.
    /// - `UARTError::DriverError`: If there is an error initializing the driver.
    pub fn set_pins_for_uart(
        &mut self,
        tx_pin: usize,
        rx_pin: usize,
        uart_num: usize,
        baudrate: u32,
        parity: Parity,
        stopbit: StopBit,
    ) -> Result<UART<'a>, UARTError> {
        let tx_peripheral = self.peripherals.get_digital_pin(tx_pin);
        let rx_peripheral = self.peripherals.get_digital_pin(rx_pin);
        let uart_peripheral = self.peripherals.get_uart(uart_num);

        UART::new(
            tx_peripheral,
            rx_peripheral,
            uart_peripheral,
            baudrate,
            parity,
            stopbit,
        )
    }

    /// Configures the BLE device as a beacon that will advertise the specified name and services.
    ///
    /// # Arguments
    ///
    /// - `advertising_name`: The name to be advertised by the BLE beacon.
    /// - `services`: A reference to a vector of services that the beacon will advertise.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `BleBeacon` instance, or an `BleError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `BleError::PeripheralError`: This error is returned if an issue occurs while initializing the BleDevice.
    /// - `BleError::ServiceDoesNotFit`: if advertising service is too big.
    /// - `BleError::Code`: To represent other errors.
    pub fn ble_beacon(
        &mut self,
        advertising_name: String,
        services: &Vec<Service>,
    ) -> Result<BleBeacon<'a>, BleError> {
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;

        BleBeacon::new(
            ble_device,
            self.get_timer_driver()?,
            advertising_name,
            services,
        )
    }

    /// Configures a BLE device as a server.
    ///
    /// # Arguments
    ///
    /// - `advertising_name`: The name to be advertised by the BLE server.
    /// - `services`: A reference to a vector of services that the server will offer.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `BleServer` instance, or an `BleError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `BleError::PeripheralError`: This error is returned if an issue occurs while initializing the BleDevice.
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property.
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server.
    pub fn ble_server(
        &mut self,
        advertising_name: String,
        services: &Vec<Service>,
    ) -> Result<BleServer<'a>, BleError> {
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;
        let ble_server = BleServer::new(
            advertising_name,
            ble_device,
            services,
            self.notification.notifier(),
            self.notification.notifier(),
        )?;
        self.interrupt_drivers.push(ble_server.get_updater());
        Ok(ble_server)
    }

    /// Configures the security settings for a BLE device.
    ///
    /// # Arguments
    ///
    /// - `ble_device`: A mutable reference to the BLEDevice instance to configure.
    /// - `security_config`: A Security configuration struct containing the desired security settings.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation completed successfully, or an `BleError` if it fails.
    ///
    /// # Errors
    ///
    /// - `BleError::InvalidParameters`: This error is returned if there is an error in the `security_config` argument.
    fn config_bluetooth_security(
        &mut self,
        ble_device: &mut BLEDevice,
        security_config: Security,
    ) -> Result<(), BleError> {
        ble_device
            .security()
            .set_auth(
                AuthReq::from_bits(security_config.auth_mode.to_le())
                    .ok_or(BleError::InvalidParameters)?,
            )
            .set_passkey(security_config.passkey)
            .set_io_cap(security_config.io_capabilities.get_code())
            .resolve_rpa();
        Ok(())
    }

    /// Configures a secure BLE server with the specified name, services, and security settings.
    ///
    /// # Arguments
    ///
    /// - `advertising_name`: The name to be advertised by the secure BLE server.
    /// - `services`: A reference to a vector of services that the server will offer.
    /// - `security_config`: A `Security` configuration struct containing the desired security settings.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `BleServer` instance, or an `BleError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `BleError::PeripheralError`: This error is returned if an issue occurs while initializing the BleDevice.
    /// - `BleError::InvalidParameters`: This error is returned if there is an error in the `security_config` argument.
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property.
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server.
    pub fn ble_secure_server(
        &mut self,
        advertising_name: String,
        services: &Vec<Service>,
        security_config: Security,
    ) -> Result<BleServer<'a>, BleError> {
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;
        self.config_bluetooth_security(ble_device, security_config)?;
        let ble_server = BleServer::new(
            advertising_name,
            ble_device,
            services,
            self.notification.notifier(),
            self.notification.notifier(),
        )?;
        self.interrupt_drivers.push(ble_server.get_updater());
        Ok(ble_server)
    }

    /// Configures a BLE client.
    /// # Returns
    ///
    /// A `Result` containing the new `BleClient` instance, or an `BleError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `BleError::PeripheralError`: This error is returned if an issue occurs while initializing the BleDevice.
    pub fn ble_client(&mut self) -> Result<BleClient, BleError> {
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;
        let ble_client = BleClient::new(ble_device, self.notification.notifier());
        self.interrupt_drivers.push(ble_client.get_updater());
        Ok(ble_client)
    }

    /// Configures a WIFIDriver. This driver uses the
    /// By default this function takes the Non-Volatile Storage of the ESP in order to save
    /// wifi configuration. This is to improve connection times for future connections
    /// to the same network.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `WifiDriver` instance, or an `WifiError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::PeripheralError`: This error is returned if an issue occurs while initializing the WifiModem.
    pub fn get_wifi_driver(&mut self) -> Result<WifiDriver<'a>, WifiError> {
        let modem = self.peripherals.get_wifi_peripheral().into_modem()?;
        WifiDriver::new(self.event_loop.clone(), modem)
    }

    /// Updates all assigned drivers of the microcontroller, handling interrupts and alarms as needed.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if all driver updates completed successfully, or an `Esp32FrameworkError` instance if it fails.
    ///
    /// # Errors
    ///
    /// If an error occurs `Esp32FrameworkError` variant is returned which corresponds to the failing update driver type.
    /// For example if a `TimerDriver` failed while updating the variant `Esp32FrameworkError::TimerDriverError` will be
    /// returned
    pub fn update(&mut self) -> Result<(), Esp32FrameworkError> {
        //timer_drivers must be updated before other drivers since this may efect the other drivers updates
        for timer_driver in &mut self.timer_drivers {
            timer_driver.update_interrupt()?;
        }
        for driver in &mut self.interrupt_drivers {
            driver.update_interrupt()?
        }
        Ok(())
    }

    /// Indefinitly blocking version of [Self::wait_for_updates]
    fn wait_for_updates_indefinitely(&mut self) {
        loop {
            self.notification.blocking_wait();
            self.update().unwrap();
        }
    }

    /// Limited blocking version of [Self::wait_for_updates]
    fn wait_for_updates_until(&mut self, miliseconds: u32) {
        let timer_driver = self.timer_drivers.first_mut().unwrap();

        let timed_out = SharableRef::new_sharable(false);
        let mut timed_out_ref = timed_out.clone();

        timer_driver.interrupt_after(miliseconds as u64 * 1000, move || {
            *timed_out_ref.deref_mut() = true
        });

        timer_driver.enable().unwrap();

        while !*timed_out.deref() {
            self.notification.blocking_wait();
            self.update().unwrap();
        }
    }

    /// Blocking function that will block for a specified time while keeping updated the microcontroller and other drivers.
    /// It is necesary to call this function from time to time, so that any interrupt that was set on any driver can be
    /// executed properly. Another way to avoid calling this function is to use an asynchronouse aproach, see [Self::block_on]
    ///
    /// # Arguments
    /// - milioseconds: Amount of miliseconds for which this function will at least block. If None is received then this
    ///   function will block for ever
    ///
    /// # Panics
    ///
    /// This function will panic if an update fails. This is considered to leave the microcontroller in an invalid state
    /// so all execution is stopped inmediatly by panicking
    pub fn wait_for_updates(&mut self, miliseconds: Option<u32>) {
        match miliseconds {
            Some(milis) => self.wait_for_updates_until(milis),
            None => self.wait_for_updates_indefinitely(),
        }
    }

    /// This will block the current thread for at least the specified amount of microseconds. Take into account this
    /// also means moast interrupts wont trigger, so if you need to block the thread while having driver interrupts
    /// take a look at [Self::wait_for_updates]
    ///
    /// # Arguments
    /// - miliseconds: The amount of miliseconds for which this function will at least block
    pub fn sleep(&self, miliseconds: u32) {
        FreeRtos::delay_ms(miliseconds)
    }

    /// Async function that will block waiting for notifications and calling [Self::update] until a signal is received.
    /// This fucntions is the concurrent task that is executed along side with the user callback in [Self::block_on]
    ///
    /// # Arguments
    /// - finished: A `SharableRef` to a `bool`, that informs us when to stop waiting. On top of setting finished as `true`
    ///   a notification must me sent to
    ///
    /// # Panics
    ///
    /// This function will panic if an update fails. This is considered to leave the microcontroller in an invalid state
    /// so all execution is stopped inmediatly by panicking
    async fn wait_for_updates_until_finished(&mut self, finished: SharableRef<bool>) {
        while !*finished.deref() {
            self.notification.wait().await;
            self.update().unwrap()
        }
    }

    /// This functions works in a similar way to the block_on function from the futures crate.
    /// It blocks the current thread until the future is finished. Aditionally it will execute concurrently
    /// another task that will make sure to keep the microcontroller and created drivers updated.
    ///
    /// # Arguments
    /// - `fut`: The future to be executed.
    ///
    /// # Errors
    ///
    /// # Panics
    ///
    /// If the concurrently updating task fails, the microcontroller is considered to be in an invalid state
    /// and will stop execution inmidiatly by panicking
    pub fn block_on<F: Future>(&mut self, fut: F) -> F::Output {
        let finished = SharableRef::new_sharable(false);
        let fut = wrap_user_future(self.notification.notifier(), finished.clone(), fut);
        block_on(join(fut, self.wait_for_updates_until_finished(finished))).0
    }
}

impl<'a> Default for Microcontroller<'a> {
    /// Creates a new instance of `Microcontroller` with default settings.
    ///
    /// This implementation calls the `new()` method to initialize the `Microcontroller`.
    ///
    /// # Returns
    ///
    /// The new Microcontroller
    ///
    /// # Panics
    ///
    /// If the take on the EspSystemEventLoop fails. This may happen if it was already taken before
    fn default() -> Self {
        Self::take()
    }
}

/// Wrapps fut into a new futere. The new future will await the original future and then will communicate when it
/// finishes by sending a notification and seting finished. Finally it returns the original future output.
///
/// # Arguments
/// - fut: `Future` to wrap
/// - notifier: After executing fut, it will send a notification threw this `Notifier`
/// - finished: A `SharableRef<bool>` that will be set to true after finishing.
///
/// # Returns
///
/// The original future output
async fn wrap_user_future<F: Future>(
    notifier: Notifier,
    mut finished: SharableRef<bool>,
    fut: F,
) -> F::Output {
    let res = fut.await;
    *finished.deref_mut() = true;
    notifier.notify();
    res
}
