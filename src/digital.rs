use esp_idf_svc::{hal::gpio::*, handle::RawHandle, sys::esp_timer_create};
use std::{ops::Deref, sync::{atomic::{AtomicBool, AtomicU8, Ordering}, Mutex}};
use esp_idf_svc::hal::timer::TimerDriver;
use std::sync::Arc;

type AtomicInterruptUpdateCode = AtomicU8;

pub struct DigitalIn<'a>{
    pin_driver: PinDriver<'a, AnyIOPin, Input>, //mepa que es mala palabra lockear pero nunca vas a lockear realmente
    timer_driver: TimerDriver<'a>,
    interrupt_type: InterruptType,
    read_interval: u32,
    interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
    user_callback: fn()->(),
    debounce_ms: Option<u32>,
}

enum InterruptUpdate{
    ExecAndEnablePin,
    EnableTimerDriver,
    TimerReached,
    UnsubscribePinDriver,
    UnsubscribeTimerDriver,
    ExecFunc,
    None
}

impl InterruptUpdate{
    fn get_code(self)-> u8{
        self as u8
    }

    fn get_atomic_code(self)-> AtomicInterruptUpdateCode{
        AtomicInterruptUpdateCode::new(self.get_code())
    }

    fn from_code(code:u8)-> Self{
        match code{
            0 => Self::ExecAndEnablePin,
            1 => Self::EnableTimerDriver,
            2 => Self::TimerReached,
            3 => Self::UnsubscribePinDriver,
            4 => Self::UnsubscribeTimerDriver,
            5 => Self::ExecFunc,
            _ => Self::None,
        }
    }
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
    pub fn new(timer_driver: TimerDriver<'a>, pin: AnyIOPin, pull_type: Pull, interrupt_type: InterruptType) -> Self { //flank default: asc
        let mut pin_driver = PinDriver::input(pin).unwrap();
        pin_driver.set_pull(pull_type).unwrap();
        pin_driver.set_interrupt_type(interrupt_type).unwrap();
        
        DigitalIn{
            pin_driver: pin_driver,
            timer_driver: timer_driver,
            interrupt_type: interrupt_type, 
            read_interval: 0, 
            interrupt_update_code: Arc::from(InterruptUpdate::None.get_atomic_code()),
            debounce_ms: None,
            user_callback: || -> () {},
        }
    }
    
    fn trigger_if_mantains_after<F: FnMut() + Send + 'static>(&mut self, time_ms:u32, mut func: F)-> impl FnMut() + Send + 'static{
        
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let after_timer_cljr = move || {
            interrupt_update_code_ref.store(InterruptUpdate::TimerReached.get_code(), Ordering::SeqCst);
        };
        
        unsafe{
            self.timer_driver.subscribe(after_timer_cljr);
        }
        self.timer_driver.set_alarm(((time_ms as u64) * self.timer_driver.tick_hz()/1000) as u64);
        
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let start_timer_cljr = move || {
            interrupt_update_code_ref.store(InterruptUpdate::EnableTimerDriver.get_code(), Ordering::SeqCst);
        };
        
        return start_timer_cljr
        
    }

    fn subscribe_trigger<F: FnMut() + Send + 'static>(&mut self, mut func: F){
        unsafe {
            self.pin_driver.subscribe(func).unwrap();
        }
        self.pin_driver.enable_interrupt().unwrap();
    }

    pub fn _trigger_on_flank<F: FnMut() + Send + 'static>(&mut self , user_callback: fn()->(), callback: F){
        self.user_callback = user_callback;
        match self.debounce_ms{
            Some(debounce_ms) => {
                let wrapper = self.trigger_if_mantains_after(debounce_ms, callback);
                self.subscribe_trigger(wrapper);
                
            },
            None => self.subscribe_trigger(callback),
        }; 
    }

    pub fn trigger_on_flank(&mut self , user_callback: fn()->()){
        let interrupt_update_code_ref= self.interrupt_update_code.clone();
        let callback = move ||{
            interrupt_update_code_ref.store(InterruptUpdate::ExecAndEnablePin.get_code(), Ordering::SeqCst);
        };
        self._trigger_on_flank(user_callback, callback);
    }
    
    pub fn trigger_on_flank_first_n_times(&mut self, mut amount_of_times: usize , user_callback: fn()->()){
        if amount_of_times == 0 {
            return
        }

        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            amount_of_times -= 1;
            if amount_of_times == 0 {
                interrupt_update_code_ref.store(InterruptUpdate::ExecFunc.get_code(), Ordering::SeqCst);
            }else{
                interrupt_update_code_ref.store(InterruptUpdate::ExecAndEnablePin.get_code(), Ordering::SeqCst);

            }
        };
        self._trigger_on_flank(user_callback, callback);
    }
    
    fn enable_timer_driver(&mut self, enable: bool){
        if enable{
            self.timer_driver.set_counter(0).unwrap();
            self.timer_driver.enable_interrupt().unwrap();
        }else{
            self.timer_driver.disable_interrupt().unwrap();
        }
        self.timer_driver.enable_alarm(enable).unwrap();
        self.timer_driver.enable(enable).unwrap();
    }

    fn timer_reached(&mut self){
        let level = match self.interrupt_type{
            InterruptType::PosEdge => Level::High,
            InterruptType::NegEdge => Level::Low,
            InterruptType::AnyEdge => todo!(),
            InterruptType::LowLevel => Level::Low,
            InterruptType::HighLevel => Level::High,
        };

        if self.pin_driver.get_level() == level{
            (self.user_callback)();
        }

        self.enable_timer_driver(false);
        self.pin_driver.enable_interrupt().unwrap();
    }

    pub fn update_interrupt(&mut self){
        let interrupt_update = InterruptUpdate::from_code(self.interrupt_update_code.load(Ordering::Acquire));
        match interrupt_update{
            InterruptUpdate::ExecAndEnablePin => {
                self.pin_driver.enable_interrupt().unwrap();
                (self.user_callback)();
            },
            InterruptUpdate::EnableTimerDriver => self.enable_timer_driver(true),
            InterruptUpdate::TimerReached => self.timer_reached(),
            InterruptUpdate::UnsubscribePinDriver => self.pin_driver.unsubscribe().unwrap(),
            InterruptUpdate::UnsubscribeTimerDriver => self.timer_driver.unsubscribe().unwrap(),
            InterruptUpdate::ExecFunc => (self.user_callback)(),
            InterruptUpdate::None => {},
        }
        self.interrupt_update_code.store(InterruptUpdate::None.get_code(), Ordering::SeqCst)
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
    
    pub fn set_debounce(&mut self, new_debounce: u32){
        self.debounce_ms = Some(new_debounce);
    }

    fn set_read_intervals(self, read_interval: u32){
        
    }
}