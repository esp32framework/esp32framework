use crate::{
    // gpio::analog_in::AnalogIn, 
    gpio::digital_in::{DigitalIn, DigitalInError, InterruptType}, microcontroller::peripherals::Peripheral, 
    utils::timer_driver::TimerDriver
};
use esp_idf_svc::hal::ledc::config::TimerConfig;

const FREQUENCY_TO_SAMPLING_RATIO: u32 = 2;

pub struct AnalogInPwm<'a> {
    digital_in: DigitalIn<'a>,
    sampling: u32
}

#[derive(Debug)]
pub enum AnalogInPwmError {
    DigitalDriverError(DigitalInError)
}

impl <'a>AnalogInPwm<'a> {
    pub fn new(timer_driver: TimerDriver<'a>, per: Peripheral, frequency_hz: u32) -> Result<Self, AnalogInPwmError> {
        let digital_in = DigitalIn::new(timer_driver, per, InterruptType::AnyEdge).map_err(|e| AnalogInPwmError::DigitalDriverError(e))?;
        Ok(AnalogInPwm {
            digital_in,
            sampling: FREQUENCY_TO_SAMPLING_RATIO * frequency_hz
        })
    }

    pub fn default(timer_driver: TimerDriver<'a>, per: Peripheral)->Result<Self, AnalogInPwmError> {
        Self::new(timer_driver, per, TimerConfig::new().frequency.into())
    }

    pub fn set_sampling(&mut self, sampling: u32){
        self.sampling = sampling
    }

    pub fn read(&self) -> f32 {
        let mut highs: u32 = 0;
        for _num in 0..(self.sampling){
            if self.digital_in.is_high(){
                highs += 1
            }
        } 
        let read_value: f32 = (highs as f32) / (self.sampling as f32);
        
        return read_value
    }

    pub fn read_percentage(&self)-> f32 {
        self.read() * 100.0
    }
}