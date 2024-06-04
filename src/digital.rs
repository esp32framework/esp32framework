use esp_idf_svc::hal::gpio::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct DigitalIn<'a>{
    pin_driver: PinDriver<'a, AnyIOPin, Input>,
    react_to: Flank,
    debounce_time: u32, //cantidad de microsegundos
    read_interval: u32,
    keep_triggering: Arc<AtomicBool>,
    subscribed: bool,
}


pub enum Flank {
    Ascending,
    Descending,
    Both,
}
/*
struct DigitalOut{
    pin: Pin,
    value: DigitalValue
}

#[derive(Eq, PartialEq)]
enum DigitalValue {
    High,
    Low
}

impl DigitalOut{
    fn new() -> DigitalOut {
        
    }
    
    fn is_high(self) -> bool{
        return self.value == DigitalValue::High
    }

    fn is_low(self) -> bool{
        return self.value == DigitalValue::Low
    }
    
    fn set_high(self) -> Result<>{
        self.value = DigitalValue::High
    }
    
    fn set_low(self) -> Result<>{
        self.value = DigitalValue::Low
    }
    
    fn toggle(self) -> Result<>{
        //self.value = 
    }
    
    // fn blink(self, frequency, duration) -> Result<>{
        
    // }
}

*/


impl <'a>DigitalIn<'a> {
    pub fn new(flank: Flank, pin: AnyIOPin, pull_type: Pull, interrupt_type: InterruptType) -> Self { //flank default: asc
        let mut digital_in = PinDriver::input(pin).unwrap();
        digital_in.set_pull(pull_type).unwrap();
        digital_in.set_interrupt_type(interrupt_type).unwrap();
        DigitalIn{
            pin_driver: digital_in, 
            react_to: flank, 
            debounce_time: 0, 
            read_interval: 0, 
            keep_triggering: Arc::new(AtomicBool::new(false)),
            subscribed: false,
        }
    }

    pub fn trigger_on_flank<F: FnMut() + Send + 'static>(&mut self , func: F){
        unsafe {
            self.pin_driver.subscribe(func).unwrap();
        }
        self.subscribed = true;
        self.keep_triggering.store(true, Ordering::Relaxed);
        self.pin_driver.enable_interrupt().unwrap();
    }
    
    pub fn trigger_on_flank_first_n_times<F: FnMut() + Send + 'static>(&mut self, mut amount_of_times: usize , mut func:F){
        if amount_of_times == 0 {
            return
        }

        let keep_triggering = self.keep_triggering.clone();
        let wrapper = move || {
            if amount_of_times == 0{
                keep_triggering.store(false, Ordering::Relaxed);
                return
            }
            amount_of_times -= 1;
            func()
        };
        self.trigger_on_flank(wrapper)
    }
    
    pub fn enable_interrupt(&mut self){
        if ! self.subscribed {
            return
        }
        if !self.keep_triggering.load(Ordering::Relaxed) {
            self.pin_driver.unsubscribe().unwrap();
            self.subscribed = false
        } else {
            self.pin_driver.enable_interrupt().unwrap();
        }
    }
    
    pub fn get_level(&self) -> Level{
        self.pin_driver.get_level()
    }    

    pub fn is_high(&self) -> bool{
        self.pin_driver.get_level() == Level::High
    }
    
    pub fn is_low(&self) -> bool{
        self.pin_driver.get_level() == Level::Low
    }
    
    fn set_debounce(self, new_debounce: u32){

    }

    fn set_read_intervals(self, read_interval: u32){
        
    }
}