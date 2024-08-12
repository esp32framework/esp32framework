use std::rc::Rc;
use esp_idf_svc::hal::adc::attenuation::adc_atten_t;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::adc::*;
use oneshot::config::AdcChannelConfig;
use oneshot::AdcChannelDriver;
use oneshot::AdcDriver;

use crate::microcontroller_src::microcontroller::SharableAdcDriver;
use crate::microcontroller_src::peripherals::Peripheral;

const MAX_DIGITAL_VAL: u16 = 4095;

#[derive(Debug)]
pub enum AnalogInError{
    MissingAdcDriver,
    InvalidPin,
    ErrorReading
}

/// Driver for receiving analog inputs from a particular pin
pub struct AnalogIn<'a>{ 
    adc_channel_driver: AnalogChannels<'a>
}

enum AnalogChannels<'a>{
    Channel0(AdcChannelDriver<'a, Gpio0, Rc<AdcDriver<'a, ADC1>>>),
    Channel1(AdcChannelDriver<'a, Gpio1, Rc<AdcDriver<'a, ADC1>>>),
    Channel2(AdcChannelDriver<'a, Gpio2, Rc<AdcDriver<'a, ADC1>>>),
    Channel3(AdcChannelDriver<'a, Gpio3, Rc<AdcDriver<'a, ADC1>>>),
    Channel4(AdcChannelDriver<'a, Gpio4, Rc<AdcDriver<'a, ADC1>>>),
    Channel5(AdcChannelDriver<'a, Gpio5, Rc<AdcDriver<'a, ADC1>>>),
    Channel6(AdcChannelDriver<'a, Gpio6, Rc<AdcDriver<'a, ADC1>>>),  
}

impl <'a> AnalogIn<'a> {
    /// Create a new _AnalogIn for a specific pin.
    pub fn new(pin: Peripheral, adc_driver: SharableAdcDriver<'a>, attenuation: adc_atten_t) -> Result<Self, AnalogInError> {
        Ok(AnalogIn {
            adc_channel_driver: AnalogIn::new_channel(pin,adc_driver,attenuation)?
        })
    }

    /// Creates a new analog channel driver for a given pin
    fn new_channel(pin: Peripheral, sharable_adc_driver: SharableAdcDriver<'a>, attenuation: adc_atten_t) -> Result<AnalogChannels<'a>, AnalogInError> {
        let mut config = AdcChannelConfig::new();
        config.attenuation = attenuation;
        config.resolution = Resolution::Resolution12Bit;
        config.calibration = true;
        let adc_channel_driver: AnalogChannels<'a> = match pin {
            Peripheral::Pin(pin_num) => match pin_num {
                0 => AnalogChannels::Channel0(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio0::new()}, &config).unwrap()),
                1 => AnalogChannels::Channel1(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio1::new()}, &config).unwrap()),
                2 => AnalogChannels::Channel2(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio2::new()}, &config).unwrap()),
                3 => AnalogChannels::Channel3(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio3::new()}, &config).unwrap()),
                4 => AnalogChannels::Channel4(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio4::new()}, &config).unwrap()),
                5 => AnalogChannels::Channel5(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio5::new()}, &config).unwrap()),
                6 => AnalogChannels::Channel6(AdcChannelDriver::new(sharable_adc_driver, unsafe {Gpio6::new()}, &config).unwrap()),
                _ => return Err(AnalogInError::InvalidPin),
            }
            _ => return Err(AnalogInError::InvalidPin),
        };
        Ok(adc_channel_driver)
    }
    
    /// Returns a digital value read from the analog pin. 
    /// The value returned is already attenuated and the range possible depends on the attenuation set.
    pub fn read(&mut self) -> Result<u16, AnalogInError> {
        let mut read_value = match self.adc_channel_driver{
            AnalogChannels::Channel0(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel1(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel2(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel3(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel4(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel5(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel6(ref mut channel_driver) => channel_driver.read(),
        }.map_err(|_| AnalogInError::ErrorReading)?;

        if read_value > MAX_DIGITAL_VAL {
            read_value = MAX_DIGITAL_VAL;
        }
        Ok(read_value)
    }
    
    //TODO: max_in_time, min_in_time, bigger_than, lower_than

    /// Returns the raw value read from an analog pin. 
    /// The value returned is not attenuated, so its ranges is [0, 4095].
    pub fn read_raw(&mut self) -> Result<u16, AnalogInError> {
        match self.adc_channel_driver{
            AnalogChannels::Channel0(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel1(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel2(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel3(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel4(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel5(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel6(ref mut channel_driver) => channel_driver.read_raw(),
        }.map_err(|_| AnalogInError::ErrorReading)
    }
    
    /// Reads *amount_of_samples* times from the analog pin and returns the average value.
    /// It is used to get a more stable value from the analog pin.
    pub fn smooth_read(&mut self, amount_of_samples: u16) -> Result<u16, AnalogInError> {
        let mut smooth_val: u16 = 0;
        for _ in 0..amount_of_samples {
            let read_val = self.read()?;
            smooth_val += read_val;
        }
        let result = smooth_val / amount_of_samples;
        Ok(result)
    }
}