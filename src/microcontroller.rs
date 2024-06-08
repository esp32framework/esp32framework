use std::collections::HashMap;
use std::sync::Arc;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::timer::{TIMER00, TimerDriver, TimerConfig};


use crate::digital::{DigitalIn, Pull, };

pub struct Microcontroller<'a>{
    peripherals: HashMap<u32, AnyIOPin>,
    timer_driver: Option<TimerDriver<'a>>,
}

impl <'a>Microcontroller<'a>{
    pub fn new() -> Self{
        esp_idf_svc::sys::link_patches();
        let (pins, timer) = get_peripherals();
        Microcontroller{
            peripherals: pins,
            timer_driver: Some(TimerDriver::new(timer, &TimerConfig::new()).unwrap()),
        }
    }

    fn _get_pin(&mut self, pin_num: u32)->AnyIOPin{
        self.peripherals.remove(&pin_num).unwrap()
    }

    pub fn set_pin_as_digital_in(&mut self, pin_num: u32, interrupt_type: InterruptType)-> DigitalIn<'a>{
        let pin = self._get_pin(pin_num);
        return DigitalIn::new(self.timer_driver.take().unwrap(), pin, interrupt_type).unwrap();
    }
    
    /*
    fn set_pin_as_digital_out(self, pin: u32) {
        let pin = _get_pin(pin);
        self.peripherals.pins.into();
        let mut digital_out = PinDriver::output(self.peripherals.pins.pin);
        digital_out(pin)
    }
    */
    
    pub fn update(&mut self, drivers: Vec<&mut DigitalIn>){
        for driver in drivers{
            driver.update_interrupt();
        }
        FreeRtos::delay_ms(10_u32);
    }
}

fn get_peripherals()->(HashMap<u32, AnyIOPin>, TIMER00){ 
    let dp = Peripherals::take().unwrap();
    let gpio9 = dp.pins.gpio9.downgrade();
    let gpio10 = dp.pins.gpio10.downgrade();
    let timer = dp.timer00;
    let mut dict: HashMap<u32, AnyIOPin> = HashMap::new();
    // inicializar todos
    dict.insert(9, gpio9);
    dict.insert(10,gpio10);
    (dict, timer)
}