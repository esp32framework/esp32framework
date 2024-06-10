use esp_idf_svc::{hal::gpio::*, sys::{EspError, ESP_ERR_INVALID_ARG, ESP_ERR_INVALID_STATE}};
use std::sync::atomic::{AtomicU8, Ordering};
use esp_idf_svc::hal::timer::TimerDriver;
use std::sync::Arc;
pub use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use crate::error_text_parser::map_enable_disable_errors;

#[derive(Debug)]
pub enum DigitalOutError{
    CannotSetPinAsOutput,
    InvalidPin
}

pub struct DigitalOut<'a>{
    pin_driver: PinDriver<'a, AnyIOPin, Output>,
}

impl <'a>DigitalOut<'a> {
    pub fn new(pin: AnyIOPin) -> Result<DigitalOut<'a>, DigitalOutError>{
        let pin_driver = PinDriver::output(pin).map_err(|_| DigitalOutError::CannotSetPinAsOutput)?;

        Ok(DigitalOut {
            pin_driver: pin_driver  
        })
    }

    pub fn set_level(&mut self, level: Level)->Result<(), DigitalOutError>{
        self.pin_driver.set_level(level).map_err(|_| DigitalOutError::InvalidPin)
    }

    pub fn get_level(&mut self) -> Level {
        if self.pin_driver.is_set_high() {
            return Level::High
        }else{
            return Level::Low
        }
    }

    pub fn set_high(&mut self)->Result<(), DigitalOutError>{
        self.set_level(Level::High)
    }
    
    pub fn set_low(&mut self)->Result<(), DigitalOutError>{
        self.set_level(Level::Low)
    }

    pub fn toggle(&mut self) ->Result<(), DigitalOutError>{
        if self.pin_driver.is_set_high(){
            self.set_level(Level::Low)
        }else{
            self.set_level(Level::High)
        }
    }
    
    /// makes the pin blink for a certain period of time blink_period (ms) and in a certain frecuency (ms)
    // pub fn blink(&mut self, blink_period: u32, frecuency: u32) -> Result<(), DigitalOutError>{

    //     self
    //     let mut time = start_time;
    //     while(time - start_time < loop_period){
    //         self.toggle();

    //         FreeRtos::delay_ms(frecuency);
    //     }
    //     Ok(())
    // }
}