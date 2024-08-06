use crate::{
    gpio::digital_in::{DigitalIn, DigitalInError}, microcontroller::peripherals::Peripheral, 
    utils::timer_driver::TimerDriver
};
use esp_idf_svc::hal::ledc::config::TimerConfig;

const FREQUENCY_TO_SAMPLING_RATIO: u32 = 2;

#[derive(Debug)]
pub enum AnalogInPwmError {
    DigitalDriverError(DigitalInError)
}

/// Driver for receiving analog input with a PWM signal from a particular DigitalIn
pub struct AnalogInPwm<'a> {
    digital_in: DigitalIn<'a>,
    sampling: u32
}

impl <'a>AnalogInPwm<'a> {
    /// Create a new AnalogInPwm for a specific pin. 
    /// The Frecuency to Sampling ratio is defined in 2 by default
    pub fn new(timer_driver: TimerDriver<'a>, per: Peripheral, frequency_hz: u32) -> Result<Self, AnalogInPwmError> {
        let digital_in = DigitalIn::new(timer_driver, per, None).map_err(AnalogInPwmError::DigitalDriverError)?;
        Ok(AnalogInPwm {
            digital_in,
            sampling: FREQUENCY_TO_SAMPLING_RATIO * frequency_hz
        })
    }

    /// Create a new AnalogInPwm with a default frecuency of 1000Hz. 
    pub fn default(timer_driver: TimerDriver<'a>, per: Peripheral)->Result<Self, AnalogInPwmError> {
        Self::new(timer_driver, per, TimerConfig::new().frequency.into())
    }

    /// Changes the amount of samples taken in a read.
    pub fn set_sampling(&mut self, sampling: u32){
        self.sampling = sampling
    }

    /// Returns the intensity value [0 , 1] obtained dividing the amount 
    /// of Highs read by the amount of samples taken.
    pub fn read(&self) -> f32 {
        let mut highs: u32 = 0;
        for _num in 0..(self.sampling){
            if self.digital_in.is_high(){
                highs += 1
            }
        } 
        (highs as f32) / (self.sampling as f32)
    }

    /// Returns the intensity value using percentage.
    pub fn read_percentage(&self)-> f32 {
        self.read() * 100.0
    }
}