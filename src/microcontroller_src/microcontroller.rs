use attenuation::adc_atten_t;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{adc::*, delay::FreeRtos, task::block_on}};
use std::rc::Rc;
use esp32_nimble::{enums::AuthReq, BLEDevice};
use futures::future::{join, Future};
use crate::{
    ble::{ble_client::BleClient, BleBeacon, BleError, BleServer, Security, Service}, 
    gpio::*,
    microcontroller_src::{interrupt_driver::InterruptDriver, peripherals::*}, 
    serial::{I2CError, I2CMaster, I2CSlave, Parity, StopBit, UARTError, UART}, 
    timer_driver::TimerDriverError, 
    utils::{
        auxiliary::{SharableRef, SharableRefExt}, 
        esp32_framework_error::{AdcDriverError, Esp32FrameworkError}, 
        notification::{Notification, Notifier}, 
        timer_driver::TimerDriver
    }, 
    wifi::{WifiDriver, WifiError}
};
use oneshot::AdcDriver;

const TIMER_GROUPS: usize = 2;

pub type SharableAdcDriver<'a> = Rc<AdcDriver<'a, ADC1>>;

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
    interrupt_drivers: Vec<Box<dyn InterruptDriver + 'a>>,
    adc_driver: Option<SharableAdcDriver<'a>>,
    notification: Notification,
    event_loop: EspSystemEventLoop,
}

impl <'a>Microcontroller<'a> {

    /// Creates a new Microcontroller instance
    /// 
    /// # Returns
    /// 
    /// The new Microcontroller
    pub fn new() -> Self{
        esp_idf_svc::sys::link_patches();
        let peripherals = Peripherals::new();
        
        Microcontroller{
            peripherals,
            timer_drivers: vec![],
            interrupt_drivers: Vec::new(),
            adc_driver: None,
            notification: Notification::new(),
            event_loop: EspSystemEventLoop::take().expect("Error creating microcontroller"),
        }
    }
    

    /// Retrieves a TimerDriver instance.
    ///
    /// # Returns
    ///
    /// A `TimerDriver` instance that can be used to manage timers in the microcontroller.
    /// If the number of existing `TimerDriver`s is less than 2, a new one is created and added to the list.
    /// Otherwise, the first driver in the list is reused.
    ///
    /// # Panics
    ///
    /// This function will panic if the `TimerDriver` cannot be created, which might happen due to hardware constraints.
    pub fn get_timer_driver(&mut self)-> Result<TimerDriver<'a>, TimerDriverError>{
        let mut timer_driver = if self.timer_drivers.len() < TIMER_GROUPS{
            let timer = self.peripherals.get_timer(self.timer_drivers.len());
            TimerDriver::new(timer, self.notification.notifier())?
        } else {
            self.timer_drivers.swap_remove(0)
        };

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
    /// A `DigitalIn` instance that can be used to read digital inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `DigitalIn` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pin_as_digital_in(&mut self, pin_num: usize) -> Result<DigitalIn<'a>, DigitalInError>  {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let dgin = DigitalIn::new(self.get_timer_driver()?, pin_peripheral, Some(self.notification.notifier()))?;
        self.interrupt_drivers.push(Box::new(dgin.clone()));
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
    /// A `DigitalOut` instance that can be used to write digital outputs to the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `DigitalOut` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pin_as_digital_out(&mut self, pin_num: usize) -> Result<DigitalOut<'a>, DigitalOutError> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let dgout = DigitalOut::new(self.get_timer_driver()?, pin_peripheral)?;
        self.interrupt_drivers.push(Box::new(dgout.clone()));
        Ok(dgout)
    }
    
    /// Starts an adc driver if no other was started before. Bitwidth is always set to 12, since 
    /// the ESP32-C6 only allows that width
    /// 
    /// # Panics
    ///
    /// This function will panic if the `AdcDriver` instance cannot be created, which might happen due to hardware constraints.
    /// 
    fn start_adc_driver(&mut self) -> Result<(), AdcDriverError>{
        if self.adc_driver.is_none() {
            self.peripherals.get_adc();
            let driver = AdcDriver::new(unsafe{ADC1::new()})?;
            self.adc_driver.replace(Rc::new(driver));
        };
        Ok(())
    }

    fn set_pin_as_analog_in(&mut self, pin_num: usize, attenuation: adc_atten_t)->Result<AnalogIn<'a>, AnalogInError>{
        self.start_adc_driver()?;
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        let adc_driver = self.adc_driver.clone().ok_or(AnalogInError::AdcDriverError(AdcDriverError::AlreadyTaken))?;
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
    /// An `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogIn` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pin_as_analog_in_low_atten(&mut self, pin_num: usize) -> Result<AnalogIn<'a>, AnalogInError>{
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
    /// An `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogIn` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration. 
    pub fn set_pin_as_analog_in_medium_atten(&mut self, pin_num: usize) -> Result<AnalogIn<'a>, AnalogInError> {
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
    /// An `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogIn` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration. 
    pub fn set_pin_as_analog_in_high_atten(&mut self, pin_num: usize) -> Result<AnalogIn<'a>, AnalogInError> {
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
    /// An `AnalogIn` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogIn` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration. 
    pub fn set_pin_as_analog_in_no_atten(&mut self, pin_num: usize) -> Result<AnalogIn<'a>, AnalogInError> {
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
    /// An `AnalogOut` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogOut` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration. 
    pub fn set_pin_as_analog_out(&mut self, pin_num: usize, freq_hz: u32, resolution: u32) ->  Result<AnalogOut<'a>, AnalogOutError>{
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        let anlg_out = AnalogOut::new(pwm_channel, pwm_timer, pin_peripheral, self.get_timer_driver()?, freq_hz, resolution)?;
        self.interrupt_drivers.push(Box::new(anlg_out.clone()));
        Ok(anlg_out)
    } 

    /// Sets pin as analog output, with a default frequency of 100 Hertz and resolution of 8 bits
    /// 
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// An `AnalogOut` instance that can be used to read analog inputs from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogOut` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration. 
    pub fn set_pin_as_default_analog_out(&mut self, pin_num: usize) -> Result<AnalogOut<'a>, AnalogOutError> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        let anlg_out = AnalogOut::default(pwm_channel, pwm_timer, pin_peripheral, self.get_timer_driver()?)?;
        self.interrupt_drivers.push(Box::new(anlg_out.clone()));
        Ok(anlg_out)
    }
    
    /// Sets pin as analog input of PWM signals, with default signal frequency of 1000 Hertz
    /// 
    /// # Arguments
    ///
    /// - `pin_num`: The number of the pin on the microcontroller to configure as an analog input.
    ///
    /// # Returns
    ///
    /// An `AnalogInPwm` instance that can be used to read analog inputs of PWM signals from the specified pin.
    ///
    /// # Panics
    ///
    /// This function will panic if the `AnalogInPwm` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pin_as_analog_in_pwm(&mut self, pin_num: usize) -> Result<AnalogInPwm<'a>, AnalogInPwmError> {
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
    /// An `I2CMaster` instance configured to use the specified SDA and SCL pins.
    ///
    /// # Panics
    ///
    /// This function will panic if the I2C peripheral is already in use or if the `I2CMaster` instance cannot be created, 
    /// which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pins_for_i2c_master(&mut self, sda_pin: usize, scl_pin: usize) -> Result<I2CMaster<'a>, I2CError> {
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
    /// An `I2CSlave` instance configured to use the specified SDA and SCL pins and the provided slave address.
    ///
    /// # Panics
    ///
    /// This function will panic if the I2C peripheral is already in use or if the `I2CSlave` instance cannot be created,
    /// which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pins_for_i2c_slave(&mut self, sda_pin: usize, scl_pin: usize, slave_addr: u8) -> Result<I2CSlave<'a>, I2CError> {
        let sda_peripheral = self.peripherals.get_digital_pin(sda_pin);
        let scl_peripheral = self.peripherals.get_digital_pin(scl_pin);
        
        I2CSlave::new(sda_peripheral, scl_peripheral, self.peripherals.get_i2c(), slave_addr)
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
    /// A `UART` instance configured with the default settings for the specified TX, RX pins, and UART number.
    ///
    /// # Panics
    ///
    /// This function will panic if the `UART` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pins_for_default_uart(&mut self, tx_pin: usize, rx_pin: usize, uart_num: usize) -> Result<UART<'a>, UARTError> {
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
    /// A `UART` instance configured with the specified settings for the TX, RX pins, and UART number.
    ///
    /// # Panics
    ///
    /// This function will panic if the `UART` instance cannot be created, which might happen due to hardware constraints or incorrect pin configuration.
    pub fn set_pins_for_uart(&mut self, tx_pin: usize, rx_pin: usize, uart_num: usize, baudrate: u32, parity: Parity, stopbit: StopBit) -> Result<UART<'a>, UARTError> {
        let tx_peripheral = self.peripherals.get_digital_pin(tx_pin);
        let rx_peripheral = self.peripherals.get_digital_pin(rx_pin);
        let uart_peripheral = self.peripherals.get_uart(uart_num);

        UART::new(tx_peripheral, rx_peripheral, uart_peripheral, baudrate, parity, stopbit)
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
    /// A `BleBeacon` instance configured to advertise the specified name and services.
    ///
    /// # Panics
    ///
    /// This function will panic if the `BleBeacon` instance cannot be created, which might happen due to hardware 
    /// constraints or incorrect configuration of the BLE device.
    pub fn ble_beacon(&mut self, advertising_name: String, services: &Vec<Service>)-> Result<BleBeacon<'a>, BleError>{
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;

        BleBeacon::new(ble_device, self.get_timer_driver()?, advertising_name, services)
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
    /// A `BleServer` instance configured to advertise the specified name and services.
    ///
    /// # Panics
    ///
    /// This function will panic if the `BleServer` instance cannot be created, which might happen due to hardware
    /// constraints or incorrect configuration of the BLE device.
    pub fn ble_server(&mut self, advertising_name: String, services: &Vec<Service>)-> Result<BleServer<'a>, BleError>{
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;
        let ble_server = BleServer::new(advertising_name, ble_device, services, self.notification.notifier(),self.notification.notifier() )?;
        self.interrupt_drivers.push(Box::new(ble_server.clone()));
        Ok(ble_server)
    }

    /// Configures the security settings for a BLE device.
    ///
    /// # Arguments
    ///
    /// - `ble_device`: A mutable reference to the BLEDevice instance to configure.
    /// - `security_config`: A Security configuration struct containing the desired security settings.
    ///
    /// # Panics
    ///
    /// This function will panic if any of the security settings cannot be applied, which might happen due to invalid configuration values.
    fn config_bluetooth_security(&mut self, ble_device: &mut BLEDevice, security_config: Security)-> Result<(), BleError>{
        ble_device.security()
        .set_auth(AuthReq::from_bits(security_config.auth_mode.to_le()).ok_or(BleError::InvalidParameters)?)
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
    /// A `BleServer` instance configured with the specified security settings, advertising name, and services.
    ///
    /// # Panics
    ///
    /// This function will panic if the `BleServer` instance cannot be created, or if the security settings cannot be applied,
    /// which might happen due to hardware constraints or incorrect configuration of the BLE device.
    pub fn ble_secure_server(&mut self, advertising_name: String, services: &Vec<Service>, security_config: Security)-> Result<BleServer<'a>, BleError> {
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;
        self.config_bluetooth_security(ble_device,security_config)?;
        let ble_server = BleServer::new(advertising_name, ble_device, services, self.notification.notifier(),self.notification.notifier())?;
        self.interrupt_drivers.push(Box::new(ble_server.clone()));
        Ok(ble_server)
    }

    pub fn ble_client(&mut self)-> Result<BleClient, BleError> {
        let ble_device = self.peripherals.get_ble_peripheral().into_ble_device()?;
        let ble_client = BleClient::new(ble_device, self.notification.notifier());
        self.interrupt_drivers.push(Box::new(ble_client.clone()));
        Ok(ble_client)
    }
    
    ///TODO: Docu of default space of nvs
    pub fn get_wifi_driver(&mut self) -> Result<WifiDriver<'a>,WifiError> {
        let modem = self.peripherals.get_wifi_peripheral().into_modem()?;
        WifiDriver::new(self.event_loop.clone(), modem)
    }

    pub fn update(&mut self)-> Result<(), Esp32FrameworkError> {
        //timer_drivers must be updated before other drivers since this may efect the other drivers updates
        for timer_driver in &mut self.timer_drivers{
            timer_driver.update_interrupt()?;
        }
        for driver in &mut self.interrupt_drivers{
            driver.update_interrupt()?
        }
        Ok(())
    }
    
    fn wait_for_updates_indefinitly(&mut self)-> Result<(), Esp32FrameworkError>{
        loop{
            self.notification.blocking_wait();
            self.update()?;
        }
    }

    fn wait_for_updates_until(&mut self, miliseconds:u32)-> Result<(), Esp32FrameworkError>{
        let timer_driver = match self.timer_drivers.first_mut(){
            Some(timer_driver) => timer_driver,
            None => &mut self.get_timer_driver().unwrap(),
        };
        
        let timed_out = SharableRef::new_sharable(false);
        let mut timed_out_ref = timed_out.clone();

        timer_driver.interrupt_after(miliseconds as u64 * 1000, move || {
            *timed_out_ref.deref_mut() = true
        });

        timer_driver.enable().unwrap();

        while !*timed_out.deref(){
            self.notification.blocking_wait();
            self.update()?;
        }
        Ok(())
    }

    pub fn wait_for_updates(&mut self, miliseconds:Option<u32>)-> Result<(), Esp32FrameworkError>{
        match miliseconds{
            Some(milis) => self.wait_for_updates_until(milis),
            None => self.wait_for_updates_indefinitly(),
        }
    }

    pub fn sleep(&self, miliseconds:u32){
        FreeRtos::delay_ms(miliseconds)
    }

    async fn wait_for_updates_until_finished(&mut self, finished: SharableRef<bool>)-> Result<(), Esp32FrameworkError>{
        while !*finished.deref(){
            self.notification.wait().await;
            self.update().map_err(|err| {
                println!("{:?}", err);
                err
            })?
        }
        Ok(())
    }

    pub fn block_on<F: Future>(&mut self, fut: F)-> Result<F::Output, Esp32FrameworkError>{
        let finished = SharableRef::new_sharable(false);
        let fut = wrap_user_future(self.notification.notifier(), finished.clone(), fut);
        let res = block_on(join(fut, self.wait_for_updates_until_finished(finished)));
        res.1?;
        Ok(res.0)
    }
}

impl<'a> Default for Microcontroller<'a> {
    fn default() -> Self {
    Self::new()
    }
}

async fn wrap_user_future<F: Future>(notifier: Notifier, mut finished: SharableRef<bool>, fut: F)-> F::Output{
    let res = fut.await;
    *finished.deref_mut() = true;
    notifier.notify();
    res
}
