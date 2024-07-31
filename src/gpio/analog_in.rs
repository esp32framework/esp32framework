use esp_idf_svc::hal::adc::attenuation::adc_atten_t;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::adc::*;

use crate::microcontroller::microcontroller::SharableAdcDriver;
use crate::microcontroller::peripherals::Peripheral;

const MAX_DIGITAL_VAL: u16 = 4095;

#[derive(Debug)]
pub enum AnalogInError{
    MissingAdcDriver,
    InvalidPin,
    ErrorReading
}

pub type AnalogInLowAtten<'a> = AnalogIn<'a, {attenuation::DB_2_5}>;
pub type AnalogInMediumAtten<'a> = AnalogIn<'a, {attenuation::DB_6}>;
pub type AnalogInHighAtten<'a> = AnalogIn<'a, {attenuation::DB_11}>;
pub type AnalogInNoAtten<'a> = AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}>;

/// Driver for receiving analog inputs from a particular pin
pub struct AnalogIn<'a, const A: adc_atten_t>{ 
    adc_channel_driver: AnalogChannels<'a, A>,
    adc_driver_ref: SharableAdcDriver<'a>,
}

enum AnalogChannels<'a, const A: adc_atten_t>{
    Channel0(AdcChannelDriver<'a, A, Gpio0>),
    Channel1(AdcChannelDriver<'a, A, Gpio1>),
    Channel2(AdcChannelDriver<'a, A, Gpio2>),
    Channel3(AdcChannelDriver<'a, A, Gpio3>),
    Channel4(AdcChannelDriver<'a, A, Gpio4>),
    Channel5(AdcChannelDriver<'a, A, Gpio5>),
    Channel6(AdcChannelDriver<'a, A, Gpio6>),  
}

impl <'a, const A: adc_atten_t> AnalogIn<'a, A> {
    /// Create a new AnalogIn for a specific pin.
    pub fn new(pin: Peripheral, adc_driver: SharableAdcDriver<'a>) -> Result<AnalogIn<'a, A>, AnalogInError> {
        {
            let driver = adc_driver.borrow_mut();
            if let None = *driver {
                return Err(AnalogInError::MissingAdcDriver)
            }
        }
        Ok(AnalogIn {
            adc_channel_driver: AnalogIn::new_channel(pin)?,
            adc_driver_ref: adc_driver,
        })
    }

    /// Creates a new analog channel driver for a given pin
    fn new_channel(pin: Peripheral) -> Result<AnalogChannels<'a, A>, AnalogInError> {
        let adc_channel_driver: AnalogChannels<'a, A> = match pin {
            Peripheral::Pin(pin_num) => match pin_num {
                0 => AnalogChannels::Channel0(AdcChannelDriver::new(unsafe {Gpio0::new()}).unwrap()),
                1 => AnalogChannels::Channel1(AdcChannelDriver::new(unsafe {Gpio1::new()}).unwrap()),
                2 => AnalogChannels::Channel2(AdcChannelDriver::new(unsafe {Gpio2::new()}).unwrap()),
                3 => AnalogChannels::Channel3(AdcChannelDriver::new(unsafe {Gpio3::new()}).unwrap()),
                4 => AnalogChannels::Channel4(AdcChannelDriver::new(unsafe {Gpio4::new()}).unwrap()),
                5 => AnalogChannels::Channel5(AdcChannelDriver::new(unsafe {Gpio5::new()}).unwrap()),
                6 => AnalogChannels::Channel6(AdcChannelDriver::new(unsafe {Gpio6::new()}).unwrap()),
                _ => return Err(AnalogInError::InvalidPin),
            }
            _ => return Err(AnalogInError::InvalidPin),
        };
        Ok(adc_channel_driver)
    }
    
    /// Returns a digital value read from the analog pin. 
    /// The value returned is already attenuated and the range possible depends on the attenuation set.
    pub fn read(&mut self) -> Result<u16, AnalogInError> {
        let mut adc_driver_ref = self.adc_driver_ref.borrow_mut();
        let mut read_value = match *adc_driver_ref{
            Some(ref mut adc_driver) => match &mut self.adc_channel_driver {
                AnalogChannels::Channel0(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
                AnalogChannels::Channel1(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
                AnalogChannels::Channel2(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
                AnalogChannels::Channel3(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
                AnalogChannels::Channel4(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
                AnalogChannels::Channel5(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
                AnalogChannels::Channel6(ref mut adc_channel_driver) => adc_driver.read(adc_channel_driver),
            }.map_err(|_| AnalogInError::ErrorReading)?,
            None => Err(AnalogInError::MissingAdcDriver)?
        };
        if read_value > MAX_DIGITAL_VAL {
            read_value = MAX_DIGITAL_VAL;
        }
        Ok(read_value)
    }
    
    //TODO: max_in_time, min_in_time, bigger_than, lower_than

    /// Returns the raw value read from an analog pin. 
    /// The value returned is not attenuated, so its ranges is [0, 4095].
    pub fn read_raw(&mut self) -> Result<u16, AnalogInError> {
        let mut adc_driver_ref = self.adc_driver_ref.borrow_mut();
        match *adc_driver_ref{
            Some(ref mut adc_driver) => match &mut self.adc_channel_driver {
                AnalogChannels::Channel0(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
                AnalogChannels::Channel1(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
                AnalogChannels::Channel2(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
                AnalogChannels::Channel3(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
                AnalogChannels::Channel4(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
                AnalogChannels::Channel5(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
                AnalogChannels::Channel6(ref mut adc_channel_driver) => adc_driver.read_raw(adc_channel_driver),
            }.map_err(|_| AnalogInError::ErrorReading),
            None => Err(AnalogInError::MissingAdcDriver)
        }
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