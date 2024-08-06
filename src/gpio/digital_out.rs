use esp_idf_svc::hal::gpio::*;
use sharable_reference_macro::sharable_reference_wrapper;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use crate::microcontroller::interrupt_driver::InterruptDriver;
use crate::utils::esp32_framework_error::Esp32FrameworkError;
use crate::utils::timer_driver::{TimerDriver,TimerDriverError};
use crate::microcontroller::peripherals::Peripheral;

type AtomicInterruptUpdateCode = AtomicU8;

#[derive(Debug)]
pub enum DigitalOutError{
    CannotSetPinAsOutput,
    InvalidPin,
    InvalidPeripheral,
    TimerDriverError(TimerDriverError)
}

/// Driver to handle a digital output for a particular Pin
pub struct _DigitalOut<'a>{
    pin_driver: PinDriver<'a, AnyIOPin, Output>,
    timer_driver: TimerDriver<'a>,
    interrupt_update_code: Arc<AtomicInterruptUpdateCode>
}

/// Driver to handle a digital output for a particular Pin
/// Wrapper of [_DigitalOut]
#[derive(Clone)]
pub struct DigitalOut<'a>{
    inner: Rc<RefCell<_DigitalOut<'a>>>
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
enum InterruptUpdate {
    Blink,
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
            0 => Self::Blink,
            _ => Self::None,
        }
    }

    fn from_atomic_code(atomic_code: Arc<AtomicInterruptUpdateCode>) -> Self {
        InterruptUpdate::from_code(atomic_code.load(Ordering::Acquire))
    }
}

#[sharable_reference_wrapper]
impl <'a>_DigitalOut<'a> {
    /// Creates a new _DigitalOut for a Pin.
    pub fn new(timer_driver: TimerDriver<'a>, per: Peripheral) -> Result<_DigitalOut<'a>, DigitalOutError>{
        let gpio = per.into_any_io_pin().map_err(|_| DigitalOutError::InvalidPeripheral)?;
        let pin_driver = PinDriver::output(gpio).map_err(|_| DigitalOutError::CannotSetPinAsOutput)?;

        Ok(_DigitalOut {
            pin_driver: pin_driver,
            timer_driver: timer_driver,
            interrupt_update_code: Arc::from(InterruptUpdate::None.get_atomic_code()),
        })
    }

    /// Sets the pin level either to High or Low
    pub fn set_level(&mut self, level: Level)->Result<(), DigitalOutError>{
        self.pin_driver.set_level(level).map_err(|_| DigitalOutError::InvalidPin)
    }

    /// Gets the current pin level
    pub fn get_level(&mut self) -> Level {
        if self.pin_driver.is_set_high() {
            Level::High
        }else{
            Level::Low
        }
    }

    /// Sets the current pin level in High
    pub fn set_high(&mut self)->Result<(), DigitalOutError>{
        self.set_level(Level::High)
    }
    
    /// Sets the current pin level in Low
    pub fn set_low(&mut self)->Result<(), DigitalOutError>{
        self.set_level(Level::Low)
    }

    /// Changes the pin level. 
    /// If the current level is High, then the pin changes its level to Low
    /// If the current level is Low, then the pin changes its level to High
    pub fn toggle(&mut self) ->Result<(), DigitalOutError>{
        if self.pin_driver.is_set_high(){
            self.set_level(Level::Low)
        }else{
            self.set_level(Level::High)
        }
    }
    
    /// Makes the pin blink for a certain amount of times defined by *amount_of_blinks*,
    /// the time states can be adjusted using *time_between_states_micro* (micro sec)
    pub fn blink(&mut self, amount_of_blinks: u32, time_between_states_micro: u64) -> Result<(), DigitalOutError> {
        let amount_of_blinks = amount_of_blinks * 2;
        if amount_of_blinks == 0 {
            return Ok(())
        }

        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            interrupt_update_code_ref.store(InterruptUpdate::Blink.get_code(), Ordering::SeqCst);
        };

        self.timer_driver.interrupt_after_n_times(time_between_states_micro, Some(amount_of_blinks), true, callback);
        self.timer_driver.enable().map_err(DigitalOutError::TimerDriverError)
    }

    /// Handles the diferent type of interrupts and reenabling the interrupt when necesary
    pub fn _update_interrupt(&mut self) -> Result<(), DigitalOutError> {
        let interrupt_update = InterruptUpdate::from_atomic_code(self.interrupt_update_code.clone());
        self.interrupt_update_code.store(InterruptUpdate::None.get_code(), Ordering::SeqCst);
        
        match interrupt_update{
            InterruptUpdate::Blink => {
                self.toggle()
            }
            InterruptUpdate::None => Ok(()),
        }
    }
}

impl<'a> DigitalOut<'a>{
    pub fn new(timer_driver: TimerDriver, per: Peripheral)->Result<DigitalOut, DigitalOutError>{
        Ok(DigitalOut{inner: Rc::new(RefCell::from(_DigitalOut::new(timer_driver, per)?))})
    }
}

#[sharable_reference_wrapper]
impl <'a> InterruptDriver for _DigitalOut<'a>{
    /// Handles the diferent type of interrupts that, executing the user callback and reenabling the 
    /// interrupt when necesary
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError> {
        self._update_interrupt().map_err(Esp32FrameworkError::DigitalOut)
    }
}