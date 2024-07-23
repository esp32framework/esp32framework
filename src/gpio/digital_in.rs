use esp_idf_svc::hal::gpio::*;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
pub use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use crate::utils::timer_driver::{TimerDriver,TimerDriverError};
use crate::utils::error_text_parser::map_enable_disable_errors;
use crate::microcontroller::peripherals::Peripheral;

type AtomicInterruptUpdateCode = AtomicU8;

#[derive(Debug)]
pub enum DigitalInError {
    CannotSetPullForPin,
    CannotSetPinAsInput,
    StateAlreadySet,
    InvalidPin,
    InvalidPeripheral,
    NoInterruptTypeSet,
    CannotSetDebounceOnAnyEdgeInterruptType,
    TimerDriverError (TimerDriverError)
}

/// Driver for receiving digital inputs from a particular Pin
pub struct DigitalIn<'a>{
    pub pin_driver: PinDriver<'a, AnyIOPin, Input>,
    timer_driver: TimerDriver<'a>,
    interrupt_type: Option<InterruptType>,
    pub interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
    user_callback: fn()->(),
    debounce_ms: Option<u64>,
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
enum InterruptUpdate {
    ExecAndEnablePin,
    EnableTimerDriver,
    TimerReached,
    UnsubscribeTimerDriver,
    ExecAndUnsubscribePin,
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
            0 => Self::ExecAndEnablePin,
            1 => Self::EnableTimerDriver,
            2 => Self::TimerReached,
            3 => Self::UnsubscribeTimerDriver,
            4 => Self::ExecAndUnsubscribePin,
            _ => Self::None,
        }
    }

    fn from_atomic_code(atomic_code: Arc<AtomicInterruptUpdateCode>) -> Self {
        InterruptUpdate::from_code(atomic_code.load(Ordering::Acquire))
    }
}

impl <'a>DigitalIn<'a> {
    /// Create a new DigitalIn for a Pin by default pull is set to Down.
    pub fn new(timer_driver: TimerDriver<'a>, per: Peripheral) -> Result<DigitalIn, DigitalInError> { //flank default: asc
        let gpio = per.into_any_io_pin().map_err(|_| DigitalInError::InvalidPeripheral)?;
        let mut pin_driver = PinDriver::input(gpio).map_err(|_| DigitalInError::CannotSetPinAsInput)?;

        let mut digital_in = DigitalIn {
            pin_driver: pin_driver,
            timer_driver: timer_driver,
            interrupt_type: None, 
            interrupt_update_code: Arc::from(InterruptUpdate::None.get_atomic_code()),
            debounce_ms: None,
            user_callback: || -> () {},
        };

        digital_in.set_pull(Pull::Down).unwrap();
        return Ok(digital_in)
    }

    /// Set the pin Pull either to Pull Up or Down
    pub fn set_pull(&mut self, pull_type: Pull)-> Result<(), DigitalInError>{
        self.pin_driver.set_pull(pull_type).map_err(|_| DigitalInError::CannotSetPullForPin)
    }

    /// Changes the interrupt type, fails if a debounce time is set and the interrupt type is AnyEdge
    pub fn change_interrupt_type(&mut self, interrupt_type: InterruptType)-> Result<(), DigitalInError>{
        if let InterruptType::AnyEdge = interrupt_type{
            return Err(DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType)
        }
        self.interrupt_type = Some(interrupt_type);
        self.pin_driver.set_interrupt_type(interrupt_type).map_err(|_| DigitalInError::InvalidPin)
    }
    
    /// After an interrupt, sets an interrupt that will trigger after an amount of microseconds. If the
    /// Level remains the same afterwards then the interrupt update is set to execute user callback
    fn trigger_if_mantains_after(&mut self, time_micro:u64)-> Result<impl FnMut() + Send + 'static, DigitalInError>{
        
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let after_timer_cljr = move || {
            interrupt_update_code_ref.store(InterruptUpdate::TimerReached.get_code(), Ordering::SeqCst);
        };

        self.timer_driver.interrupt_after(time_micro, after_timer_cljr).map_err(|err| DigitalInError::TimerDriverError(err))?;
        
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let start_timer_cljr = move || {
            interrupt_update_code_ref.store(InterruptUpdate::EnableTimerDriver.get_code(), Ordering::SeqCst);
        };
        
        Ok(start_timer_cljr)
    }

    /// Subscribes the function to the pin driver interrupt and enables it
    fn subscribe_trigger<F: FnMut() + Send + 'static>(&mut self, func: F) -> Result<(), DigitalInError>{
        unsafe {
            self.pin_driver.subscribe(func).map_err(|err| map_enable_disable_errors(err))?;
        }
        self.pin_driver.enable_interrupt().map_err(|err| map_enable_disable_errors(err))
    }
    
    /// Sets a callback that sets an InterruptUpdate on the received interrupt type, which will then
    /// execute the user callback. If a debounce is set then the level must be mantained for the
    /// user callback to be executed.
    pub fn _trigger_on_interrupt<F: FnMut() + Send + 'static>(&mut self , user_callback: fn()->(), callback: F, interrupt_type: InterruptType) -> Result<(), DigitalInError>{
        self.change_interrupt_type(interrupt_type)?;
        self.user_callback = user_callback;
        match self.debounce_ms{
            Some(debounce_ms) => {
                let wrapper = self.trigger_if_mantains_after(debounce_ms)?;
                self.subscribe_trigger(wrapper)
            },
            None => self.subscribe_trigger(callback),
        }
    }
    
    /// Sets a callback that sets an InterruptUpdate on the received interrupt type, which will then
    /// execute the user callback. If a debounce is set then the level must be mantained for the
    /// user callback to be executed.
    pub fn trigger_on_interrupt(&mut self , user_callback: fn()->(), interrupt_type: InterruptType)->Result<(), DigitalInError>{
        let interrupt_update_code_ref= self.interrupt_update_code.clone();
        let callback = move ||{
            interrupt_update_code_ref.store(InterruptUpdate::ExecAndEnablePin.get_code(), Ordering::SeqCst);
        };
        self._trigger_on_interrupt(user_callback, callback, interrupt_type)
    }
    
    /// Sets a callback to be triggered only n times before unsubscribing the interrupt.
    pub fn trigger_on_interrupt_first_n_times(&mut self, mut amount_of_times: usize , user_callback: fn()->(), interrupt_type: InterruptType) -> Result<(), DigitalInError> {
        if amount_of_times == 0 {
            return Ok(())
        }
        
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            amount_of_times -= 1;
            if amount_of_times == 0 {
                interrupt_update_code_ref.store(InterruptUpdate::ExecAndUnsubscribePin.get_code(), Ordering::SeqCst);
            }else{
                interrupt_update_code_ref.store(InterruptUpdate::ExecAndEnablePin.get_code(), Ordering::SeqCst);
                
            }
        };
        self._trigger_on_interrupt(user_callback, callback, interrupt_type)
    }

    /// Checks if the level corresponds to the set interrupt type. If it does it means the level didnt
    /// change from the messurement before the debounce time, so the user callback is executed
    fn timer_reached(&mut self) -> Result<(), DigitalInError>{
        let level = match self.interrupt_type {
            Some(InterruptType::PosEdge) => Level::High,
            Some(InterruptType::NegEdge) => Level::Low,
            Some(InterruptType::AnyEdge) => Err(DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType)?, 
            Some(InterruptType::LowLevel) => Level::Low,
            Some(InterruptType::HighLevel) => Level::High,
            None => Err(DigitalInError::NoInterruptTypeSet)?,
        };
        
        if self.pin_driver.get_level() == level{
            (self.user_callback)();
        }
        
        self.timer_driver.disable().map_err(|err| DigitalInError::TimerDriverError(err))?;
        self.pin_driver.enable_interrupt().map_err(|err| map_enable_disable_errors(err))
    }

    /// Handles the diferent type of interrupts that, executing the user callback and reenabling the 
    /// interrupt when necesary
    pub fn update_interrupt(&mut self)-> Result<(), DigitalInError>{
        let interrupt_update = InterruptUpdate::from_atomic_code(self.interrupt_update_code.clone());
        self.interrupt_update_code.store(InterruptUpdate::None.get_code(), Ordering::SeqCst);

        match interrupt_update {
            InterruptUpdate::ExecAndEnablePin => {
                (self.user_callback)();
                self.pin_driver.enable_interrupt().map_err(|err| map_enable_disable_errors(err))
            },
            InterruptUpdate::EnableTimerDriver => self.timer_driver.enable().map_err(|err| DigitalInError::TimerDriverError(err)),
            InterruptUpdate::TimerReached => self.timer_reached(),
            InterruptUpdate::ExecAndUnsubscribePin => {
                (self.user_callback)();
                self.pin_driver.unsubscribe().map_err(|err| map_enable_disable_errors(err))
                },
            InterruptUpdate::UnsubscribeTimerDriver => self.timer_driver.unsubscribe().map_err(|err| DigitalInError::TimerDriverError(err)),
            InterruptUpdate::None => Ok(()),
        }
    }
    
    /// Gets the current pin level
    pub fn get_level(&self) -> Level{
        self.pin_driver.get_level()
    }
    
    /// verifies if the pin level is High
    pub fn is_high(&self) -> bool{
        self.pin_driver.get_level() == Level::High
    }
    
    /// verifies if the pin level is Low
    pub fn is_low(&self) -> bool{
        self.pin_driver.get_level() == Level::Low
    }
    
    /// Sets the debounce time to an amount of microseconds. This means that if an interrupt is set,
    /// then the level must be the same after the debounce time for the user callback to be executed.
    /// Debounce time does not work with InterruptType::AnyEdge, an error will be returned
    pub fn set_debounce(&mut self, time_micro: u64)->Result<(), DigitalInError>{
        match self.interrupt_type{
            Some(InterruptType::AnyEdge) => Err(DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType)?,
            _ => self.debounce_ms = Some(time_micro),
        }
        Ok(())
    }
}