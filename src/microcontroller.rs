use std::collections::HashMap;
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


use crate::digital_in::{DigitalIn, Pull, InterruptType};
use crate::digital_out::DigitalOut;
use crate::timer_driver::TimerDriver;
use crate::analog_in:: AnalogIn;
use crate::peripherals::Peripherals;

pub struct Microcontroller<'a> {
    peripherals: Peripherals,
    timer_driver: Vec<TimerDriver<'a>>,
    adc_driver: Option<AdcDriver<'a, ADC1>>,
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
            adc_driver: None,
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
    
    fn start_adc_driver(&mut self, resolution: Resolution) {
        if let None = self.adc_driver{
            self.peripherals.get_adc();
            self.adc_driver = Some(AdcDriver::new(unsafe{ADC1::new()}, &Config::new().resolution(resolution).calibration(true)).unwrap());
        }
    }
    
    pub fn set_pin_as_analog_in_low_atten(&'a mut self, pin_num: usize, resolution: Resolution, attenuation: attenuation::adc_atten_t) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_2_5}> {
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_2_5}>::new(pin_peripheral, &mut self.adc_driver).unwrap()
    }

    pub fn set_pin_as_analog_in_medium_atten(&'a mut self, pin_num: usize, resolution: Resolution, attenuation: attenuation::adc_atten_t) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_6}> {
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_6}>::new(pin_peripheral, &mut self.adc_driver).unwrap()
    }

    pub fn set_pin_as_analog_in_high_atten(&'a mut self, pin_num: usize, resolution: Resolution, attenuation: attenuation::adc_atten_t) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_11}> {
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_11}>::new(pin_peripheral, &mut self.adc_driver).unwrap()
    }
    
    pub fn set_pin_as_analog_in_no_atten(&'a mut self, pin_num: usize, resolution: Resolution, attenuation: attenuation::adc_atten_t) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}> {
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}>::new(pin_peripheral, &mut self.adc_driver).unwrap()
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

// fn get_peripherals()->(HashMap<u32, AnyIOPin>, (TIMER00, TIMER10), ADC1){ 
//     let dp = Peripherals::take().unwrap();
//     let mut dict: HashMap<u32, AnyIOPin> = HashMap::new();
//     dict.insert(0, dp.pins.gpio0.downgrade());
//     dict.insert(1, dp.pins.gpio1.downgrade());
//     dict.insert(2, dp.pins.gpio2.downgrade());
//     dict.insert(3, dp.pins.gpio3.downgrade());
//     dict.insert(4, dp.pins.gpio4.downgrade());
//     dict.insert(5, dp.pins.gpio5.downgrade());
//     dict.insert(6, dp.pins.gpio6.downgrade());
//     dict.insert(7, dp.pins.gpio7.downgrade());
//     dict.insert(8, dp.pins.gpio8.downgrade());
//     dict.insert(9, dp.pins.gpio9.downgrade());
//     dict.insert(10, dp.pins.gpio10.downgrade());
//     dict.insert(11, dp.pins.gpio11.downgrade());
//     dict.insert(12, dp.pins.gpio12.downgrade());
//     dict.insert(13, dp.pins.gpio13.downgrade());
//     dict.insert(15, dp.pins.gpio15.downgrade());
//     dict.insert(16, dp.pins.gpio16.downgrade());
//     dict.insert(17, dp.pins.gpio17.downgrade());
//     dict.insert(18, dp.pins.gpio18.downgrade());
//     dict.insert(19, dp.pins.gpio19.downgrade());
//     dict.insert(20, dp.pins.gpio20.downgrade());
//     dict.insert(21, dp.pins.gpio21.downgrade());
//     let timers = (dp.timer00, dp.timer10);
//     let adc = dp.adc1;
//     (dict, timers, adc)
// }