use std::pin::Pin;

use esp_idf_svc::hal::adc::attenuation::adc_atten_t;
use esp_idf_svc::hal::adc;
use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::peripherals;
use esp_idf_svc::sys::adc_bitwidth_t;
use crate::peripherals::Peripheral;

// Atenuacion es DB
// Resolucion son bits
// const DEFAULT_RESOLUTION: u16 = ADC_BITWIDTH_DEFAULT ;
// const DEFAULT_WIDTH: u16 = ADC_BITWIDTH_DEFAULT;

pub struct AnalogIn<'a, const A: adc_atten_t, ADC: Adc>{ 
    adc_channel_driver: AnalogChannels<'a, A>,
    adc_driver_ref: &'a mut Option<AdcDriver<'a, ADC>>,
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

enum Attenuation{
    High,
    Intermidiate,
    Low,
    None
}

#[derive(Debug)]
pub enum AnalogInError{
    MissingAdcDriver,
    InvalidPin
}

impl <'a, const A: adc_atten_t, ADC: Adc> AnalogIn<'a, A, ADC> {
    pub fn new(pin: Peripheral, adc_driver: &'a mut Option<AdcDriver<'a, ADC>>) -> Result<AnalogIn<'a, A, ADC>, AnalogInError> {
        
        if let None = adc_driver {
            return Err(AnalogInError::MissingAdcDriver)
        }
        
        Ok(AnalogIn {
            adc_channel_driver: AnalogIn::<A, ADC>::new_channel(pin)?,
            adc_driver_ref: adc_driver,
        })
    }

    pub fn new_channel<const B: adc_atten_t>(pin: Peripheral) -> Result<AnalogChannels<'a, B>, AnalogInError> {
        let mut adc_channel_driver: AnalogChannels<'a, B> = match pin {
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
    

    fn digital_read() {

    }

    fn digital_write() {

    }

    fn smooth_digital_read(_samples: u32) {
        // Se lee samples veces, se suma todo y se divide por sample
    }
}