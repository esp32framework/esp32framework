use crate::{
    analog_in::AnalogIn, 
    digital_in::{DigitalIn, DigitalInError, InterruptType}, 
    timer_driver::TimerDriver,
    peripherals::Peripheral,
};

const FREQUENCY_TO_SAMPLING_RATIO: u32 = 2;


pub struct AnalogInPwm<'a> {
    digital_in: DigitalIn<'a>,
    sampling: u32
}

enum AnalogInPwmError {
    DigitalDriverError(DigitalInError)
}

impl <'a>AnalogInPwm<'a> {
    pub fn new(timer_driver: TimerDriver<'a>, per: Peripheral, interrupt_type: InterruptType, frequency_hz: u32) -> Result<Self, AnalogInPwmError> {
        let digital_in = DigitalIn::new(timer_driver, per, interrupt_type).map_err(|e| AnalogInPwmError::DigitalDriverError(e))?;
        Ok(AnalogInPwm {
            digital_in,
            sampling: FREQUENCY_TO_SAMPLING_RATIO * frequency_hz
        })
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