use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicU8,Ordering},
        Arc
    },
};

use esp_idf_svc::hal::timer;
use crate::utils::timer_driver::timer::TimerConfig;
use crate::microcontroller::peripherals::Peripheral;

type AtomicInterruptUpdateCode = AtomicU8;

#[derive(Clone)]
pub struct TimerDriver<'a> {
    inner: Rc<RefCell<_TimerDriver<'a>>>
}

struct _TimerDriver<'a> {
    driver: timer::TimerDriver<'a>,
    interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
    interrupt_callback: Box<dyn FnMut()>,
}

#[derive(Debug)]
pub enum TimerDriverError {
    CouldNotSetTimer,
    InvalidTimer,
    CannotSetTimerCounter,
    SubscriptionError
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
#[derive(Debug)]
enum InterruptUpdate {
    FinishedTriggering,
    KeepTriggering,
    None
}

impl InterruptUpdate{
    fn get_code(self)-> u8{
        self as u8
    }

    fn get_atomic_code(self)-> AtomicInterruptUpdateCode{
        AtomicInterruptUpdateCode::new(self.get_code())
    }

    fn from_code(code:u8)-> Self {
        match code{
            0 => Self::FinishedTriggering,
            1 => Self::KeepTriggering,
            _ => Self::None,
        }
    }

    fn from_atomic_code(atomic_code: Arc<AtomicInterruptUpdateCode>) -> Self {
        InterruptUpdate::from_code(atomic_code.load(Ordering::Acquire))
    }
}

impl <'a>_TimerDriver<'a>{
    //pub fn new<T: timer::Timer>(timer: impl Peripheral<P = T> + 'a)->Result<TimerDriver<'a>, TimerDriverError> {
    fn new(timer: Peripheral) -> Result<_TimerDriver<'a>, TimerDriverError> {
        let driver = match timer{
            Peripheral::Timer(timer_num) => 
                match timer_num{
                    0 => timer::TimerDriver::new(unsafe{timer::TIMER00::new()}, &TimerConfig::new()),
                    1 => timer::TimerDriver::new(unsafe{timer::TIMER10::new()}, &TimerConfig::new()),
                    _ => return Err(TimerDriverError::InvalidTimer),
                }.map_err(|_| TimerDriverError::InvalidTimer)?,
            _ => return Err(TimerDriverError::InvalidTimer),
        };

        Ok(_TimerDriver{
            driver, 
            interrupt_update_code: Arc::from(InterruptUpdate::None.get_atomic_code()),
            interrupt_callback: Box::new(|| -> () {}),
        })
    }
    
    fn interrupt_after<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u32, callback: F)-> Result<(), TimerDriverError>{
        unsafe{
            self.driver.subscribe(callback).map_err(|_| TimerDriverError::SubscriptionError)?;
        }
        self.driver.set_alarm(((micro_seconds as u64) * self.driver.tick_hz()/1000000) as u64).map_err(|_| TimerDriverError::CouldNotSetTimer)
    }
    
    fn interrupt_after_n_times<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u32, mut amount_of_triggers: Option<u32>, callback: F)-> Result<(), TimerDriverError>{
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        self.interrupt_callback = Box::new(callback);
        println!("Seting interrupt");
        let alarm_callback = move || {
            match amount_of_triggers{
                Some(ref mut remaining_triggers) => 
                    if *remaining_triggers == 0 {
                        interrupt_update_code_ref.store(InterruptUpdate::FinishedTriggering.get_code(), Ordering::SeqCst);
                    }else{
                        *remaining_triggers -= 1;
                        interrupt_update_code_ref.store(InterruptUpdate::KeepTriggering.get_code(), Ordering::SeqCst);
                    },
                None => interrupt_update_code_ref.store(InterruptUpdate::FinishedTriggering.get_code(), Ordering::SeqCst),
            };
        };

        unsafe{
            self.driver.subscribe(alarm_callback).map_err(|_| TimerDriverError::SubscriptionError)?;
        }
        self.driver.set_alarm(((micro_seconds as u64) * self.driver.tick_hz()/1000000) as u64).map_err(|_| TimerDriverError::CouldNotSetTimer)
    }

    fn _enable(&mut self, enable: bool) -> Result<(),TimerDriverError>{
        if enable{
            self.driver.set_counter(0).map_err(|_| TimerDriverError::CannotSetTimerCounter)?; 
            self.driver.enable_interrupt().map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }else{
            self.driver.disable_interrupt().map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }
        self.driver.enable_alarm(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        self.driver.enable(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        Ok(())
    }

    fn enable(&mut self) -> Result<(),TimerDriverError>{
        self._enable(true)
    }
    
    fn disable(&mut self) -> Result<(),TimerDriverError>{
        self._enable(false)
    }

    fn unsubscribe(&mut self)  -> Result<(),TimerDriverError> {
        self.driver.unsubscribe().map_err(|_| TimerDriverError::SubscriptionError)
    }

    /// Handles the diferent type of interrupts and reenabling the interrupt when necesary
    pub fn update_interrupts(&mut self) -> Result<(), TimerDriverError> {
        let interrupt_update = InterruptUpdate::from_atomic_code(self.interrupt_update_code.clone());
        self.interrupt_update_code.store(InterruptUpdate::None.get_code(), Ordering::SeqCst);
        
        match interrupt_update{
            InterruptUpdate::FinishedTriggering => {
                self.disable()?;
                self.unsubscribe()
            },
            InterruptUpdate::KeepTriggering => {
                println!("Por ejecutar el callback");
                (self.interrupt_callback)();
                self.enable()
            },
            InterruptUpdate::None => Ok(()),
        }
    }
}

impl <'a>TimerDriver<'a>{
    //pub fn new<T: timer::Timer>(timer: impl Peripheral<P = T> + 'a)->Result<TimerDriver<'a>, TimerDriverError> {
    pub fn new(timer: Peripheral) -> Result<TimerDriver<'a>, TimerDriverError> {
        Ok(TimerDriver{inner: Rc::new(RefCell::new(_TimerDriver::new(timer)?))})
    }
    
    pub fn interrupt_after<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u32, callback: F)-> Result<(), TimerDriverError>{
        self.inner.borrow_mut().interrupt_after(micro_seconds, callback)
    }
    
    pub fn interrupt_after_n_times<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u32, amount_of_triggers: Option<u32>, callback: F)-> Result<(), TimerDriverError>{
        self.inner.borrow_mut().interrupt_after_n_times(micro_seconds, amount_of_triggers, callback)
    }

    pub fn enable(&mut self) -> Result<(),TimerDriverError>{
        self.inner.borrow_mut().enable()
    }
    
    pub fn disable(&mut self) -> Result<(),TimerDriverError>{
        self.inner.borrow_mut().disable()
    }
    
    pub fn unsubscribe(&mut self)  -> Result<(),TimerDriverError> {
        self.inner.borrow_mut().unsubscribe()
    }

    pub fn update_interrupts(&mut self) -> Result<(), TimerDriverError> {
        self.inner.borrow_mut().update_interrupts()
    }
}
