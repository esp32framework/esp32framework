use esp_idf_svc::hal::adc::attenuation::adc_atten_t;
use esp_idf_svc::hal::adc;
use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::gpio::{ADCPin, AnyIOPin};
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::adc_bitwidth_t;

// Atenuacion es DB
// Resolucion son bits
// const DEFAULT_RESOLUTION: u16 = ADC_BITWIDTH_DEFAULT ;
// const DEFAULT_WIDTH: u16 = ADC_BITWIDTH_DEFAULT;

pub struct AnalogIn<'a, const A: adc_atten_t, P: ADCPin, ADC: Adc>{ 
    adc_channel_driver: AdcChannelDriver<'a, A, P>,
    adc_driver_ref: &'a mut AdcDriver<'a, ADC>,
    //adc_channel: ???, En el ESP32-C6 hay 6 channels. GPIO0 a GPIO6
}

enum Attenuation{
    High,
    Intermidiate,
    Low,
    None
}

enum AnalogInError{
    InvalidPin,
}

impl <'a, const A: adc_atten_t, P: ADCPin, ADC: Adc> AnalogIn<'a, A, P, ADC> {
    pub fn new(adc_pin: P, attenuation: adc_atten_t, adc_driver: &'a mut AdcDriver<'a, ADC>) -> Result<Self, AnalogInError>{
        
        let mut adc_channel_driver: AdcChannelDriver<A, P> =
        AdcChannelDriver::new(adc_pin).unwrap();
        Ok(AnalogIn {
            adc_channel_driver: adc_channel_driver,
            adc_driver_ref: adc_driver,
        })
        
    }

    fn digital_read() {

    }

    fn digital_write() {

    }

    fn smooth_digital_read(_samples: u32) {
        // Se lee samples veces, se suma todo y se divide por sample
    }
}