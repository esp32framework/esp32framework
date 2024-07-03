use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use attenuation::adc_atten_t;
use config::Resolution;
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::timer::{TIMER00, TIMER10};
use std::cell::RefCell;

pub type SharableAdcDriver<'a> = Rc<RefCell<Option<AdcDriver<'a, ADC1>>>>;

use crate::digital_in::{DigitalIn, Pull, InterruptType};
use crate::digital_out::DigitalOut;
use crate::timer_driver::TimerDriver;
use crate::analog_in::AnalogIn;
use crate::analog_out::AnalogOut;
use crate::peripherals::Peripherals;

pub struct Microcontroller<'a> {
    peripherals: Peripherals,
    timer_driver: Vec<TimerDriver<'a>>,
    adc_driver: SharableAdcDriver<'a>,
}

impl <'a>Microcontroller<'a>{
    pub fn new() -> Self {
        esp_idf_svc::sys::link_patches();
        let mut peripherals = Peripherals::new();
        let timer0 = peripherals.get_timer(0);
        let timer1 = peripherals.get_timer(1);
        
        Microcontroller{
            peripherals: peripherals,
            timer_driver: vec![TimerDriver::new(timer0).unwrap(), TimerDriver::new(timer1).unwrap()],
            adc_driver: Rc::new(RefCell::new(None)),
        }
    }

    pub fn set_pin_as_digital_in(&mut self, pin_num: usize, interrupt_type: InterruptType)-> DigitalIn<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        DigitalIn::new(self.timer_driver.pop().unwrap(), pin_peripheral, interrupt_type).unwrap()
    }
    
    
    pub fn set_pin_as_digital_out(&mut self, pin_num: usize) -> DigitalOut<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        DigitalOut::new(pin_peripheral, self.timer_driver.pop().unwrap()).unwrap()
    }
    
    /// Starts an adc driver if no other was started before. Bitwidth is always set to 12, since 
    /// the ESP32-C6 only allows that width
    fn start_adc_driver(&mut self) {
        let mut adc_driver = self.adc_driver.borrow_mut();
        if let None = *adc_driver {
            self.peripherals.get_adc();
            
            let driver = AdcDriver::new(unsafe{ADC1::new()}, &Config::new().resolution(Resolution::Resolution12Bit).calibration(true)).unwrap();
            adc_driver.replace(driver); 
        }
    }

    /// Sets pin as analog input with attenuation set to 2.5dB
    pub fn set_pin_as_analog_in_low_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_2_5}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_2_5}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }
    
    /// Sets pin as analog input with attenuation set to 6dB  
    pub fn set_pin_as_analog_in_medium_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_6}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_6}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }
    
    /// Sets pin as analog input with attenuation set to 11dB  
    pub fn set_pin_as_analog_in_high_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_11}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_11}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }

    /// Sets pin as analog input with attenuation set to 0dB  
    pub fn set_pin_as_analog_in_no_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }

    pub fn set_pin_as_analog_out(&mut self, pin_num: usize, freq_hz: u32, resolution: u32) -> AnalogOut<'a> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        AnalogOut::<'a>::new(pwm_channel, pwm_timer, pin_peripheral, freq_hz, resolution).unwrap()
    } 

    pub fn set_pin_as_default_analog_out(&mut self, pin_num: usize, freq_hz: u32, resolution: u32) -> AnalogOut<'a> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        AnalogOut::<'a>::default(pwm_channel, pwm_timer, pin_peripheral).unwrap()
    }

    // pub fn set_pin_as_analog_in<const A: adc_atten_t, ADC: Adc>(&mut self, pin_num: usize, resolution: Resolution, attenuation: attenuation::adc_atten_t) -> AnalogIn<'a, A, ADC> {
    //     let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
    //     self.start_adc_driver(resolution);
        
    //     AnalogIn::new(pin_peripheral,attenuation, &mut self.adc_driver.unwrap()).unwrap()
    // }
    
    //pub fn set_pin_as_analog_out()
    
    pub fn update(&mut self, drivers_in: Vec<&mut DigitalIn>, drivers_out: Vec<&mut DigitalOut>){
        for driver in drivers_in{
            driver.update_interrupt();
        }
        for driver in drivers_out{
            driver.update_interrupt();
        }
        FreeRtos::delay_ms(10_u32);
    }
}