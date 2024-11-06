use crate::{
    microcontroller_src::interrupt_driver::InterruptDriver,
    microcontroller_src::peripherals::{Peripheral, PeripheralError},
    utils::{
        auxiliary::{SharableRef, SharableRefExt},
        esp32_framework_error::Esp32FrameworkError,
        timer_driver::{TimerDriver, TimerDriverError},
    },
};
use esp_idf_svc::hal::gpio::*;
use sharable_reference_macro::sharable_reference_wrapper;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

type AtomicInterruptUpdateCode = AtomicU8;

/// Enums the different errors possible when working with BLE
#[derive(Debug)]
pub enum DigitalOutError {
    CannotSetPinAsOutput,
    InvalidPin,
    InvalidPeripheral(PeripheralError),
    TimerDriverError(TimerDriverError),
}

/// Driver to handle a digital output for a particular Pin
/// - `pin_driver`: A PinDriver instance that handles the output signals
/// - `timer_driver`: A TimerDriver instance
/// - `interrupt_update_code`: An `Arc<AtomicInterruptUpdateCode>` to handle interrupts
struct _DigitalOut<'a> {
    pin_driver: PinDriver<'a, AnyIOPin, Output>,
    timer_driver: TimerDriver<'a>,
    interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
}

/// Driver to handle a digital output for a particular Pin
pub struct DigitalOut<'a> {
    inner: SharableRef<_DigitalOut<'a>>,
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
enum InterruptUpdate {
    Blink,
    None,
}

impl InterruptUpdate {
    /// Gets the interrupt update code as a `u8` value.
    ///
    /// # Returns
    ///
    /// A `u8` value representing the interrupt update code.
    fn get_code(self) -> u8 {
        self as u8
    }

    /// Converts the interrupt update code to an `AtomicInterruptUpdateCode`.
    ///
    /// # Returns
    ///
    /// An `AtomicInterruptUpdateCode` initialized with the current interrupt update code.
    fn get_atomic_code(self) -> AtomicInterruptUpdateCode {
        AtomicInterruptUpdateCode::new(self.get_code())
    }

    /// Creates an `InterruptUpdate` instance from a given `u8` code.
    ///
    /// # Arguments
    ///
    /// - `code`: An `u8` value representing the interrupt update code.
    ///
    /// # Returns
    ///
    /// An `InterruptUpdate` instance corresponding to the provided `code`.
    fn from_code(code: u8) -> Self {
        match code {
            x if x == Self::Blink.get_code() => Self::Blink,
            _ => Self::None,
        }
    }

    /// Creates an `InterruptUpdate` instance from an `AtomicInterruptUpdateCode`.
    ///
    /// # Arguments
    ///
    /// - `atomic_code`: An `Arc<AtomicInterruptUpdateCode>` representing the atomic interrupt update code.
    ///
    /// # Returns
    ///
    /// An `InterruptUpdate` instance corresponding to the provided `atomic_code`.
    fn from_atomic_code(atomic_code: &Arc<AtomicInterruptUpdateCode>) -> Self {
        InterruptUpdate::from_code(atomic_code.load(Ordering::Acquire))
    }
}

#[sharable_reference_wrapper]
impl<'a> _DigitalOut<'a> {
    /// Creates a new `_DigitalOut` for a specified pin.
    ///
    /// # Arguments
    ///
    /// - `timer_driver`: A `TimerDriver<'a>` instance to manage the timing operations.
    /// - `per`: A `Peripheral` that can be transformed into an AnyIOPin.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_DigitalOut` instance, or a `DigitalOutError` if the initialization fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPeripheral`: If the peripheral cannot be converted into an AnyIOPin.
    /// - `DigitalOutError::CannotSetPinAsOutput`: If the pin cannot be set as an output.
    fn new(
        timer_driver: TimerDriver<'a>,
        per: Peripheral,
    ) -> Result<_DigitalOut<'a>, DigitalOutError> {
        let gpio = per
            .into_any_io_pin()
            .map_err(DigitalOutError::InvalidPeripheral)?;
        let pin_driver =
            PinDriver::output(gpio).map_err(|_| DigitalOutError::CannotSetPinAsOutput)?;

        Ok(_DigitalOut {
            pin_driver,
            timer_driver,
            interrupt_update_code: Arc::from(InterruptUpdate::None.get_atomic_code()),
        })
    }

    /// Sets the pin level to either `High` or `Low`.
    ///
    /// # Arguments
    ///
    /// - `level`: A Level value to set the pin to.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalOutError` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPin`: If the pin level cannot be set.
    pub fn set_level(&mut self, level: Level) -> Result<(), DigitalOutError> {
        self.pin_driver
            .set_level(level)
            .map_err(|_| DigitalOutError::InvalidPin)
    }

    /// Gets the current level of the pin.
    ///
    /// # Returns
    ///
    /// A `Level` indicating whether the pin is `High` or `Low`.
    pub fn get_level(&mut self) -> Level {
        if self.pin_driver.is_set_high() {
            Level::High
        } else {
            Level::Low
        }
    }

    /// Sets the pin level to `High`.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalOutError` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPin`: If the pin level cannot be set.
    pub fn set_high(&mut self) -> Result<(), DigitalOutError> {
        self.set_level(Level::High)
    }

    /// Sets the pin level to `Low`.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalOutError` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPin`: If the pin level cannot be set.
    pub fn set_low(&mut self) -> Result<(), DigitalOutError> {
        self.set_level(Level::Low)
    }

    /// Changes the pin level.
    /// If the current level is High, then the pin changes its level to Low
    /// If the current level is Low, then the pin changes its level to High
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalOutError` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPin`: If the pin level cannot be toggled.
    pub fn toggle(&mut self) -> Result<(), DigitalOutError> {
        if self.pin_driver.is_set_high() {
            self.set_level(Level::Low)
        } else {
            self.set_level(Level::High)
        }
    }

    /// Makes the pin blink for a certain amount of times defined by *amount_of_blinks*,
    /// the time states can be adjusted using *time_between_states_micro* (micro sec)
    ///
    /// # Arguments
    ///
    /// * `amount_of_blinks` - Amount of times the pin will blink
    /// * `time_between_states_micro` - Time between each state change in micro seconds
    ///
    /// # Example
    ///
    ///  
    pub fn blink(
        &mut self,
        amount_of_blinks: u32,
        time_between_states_micro: u64,
    ) -> Result<(), DigitalOutError> {
        let amount_of_blinks = amount_of_blinks * 2;
        if amount_of_blinks == 0 {
            return Ok(());
        }

        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            interrupt_update_code_ref.store(InterruptUpdate::Blink.get_code(), Ordering::SeqCst);
        };

        self.timer_driver.interrupt_after_n_times(
            time_between_states_micro,
            Some(amount_of_blinks - 1),
            true,
            callback,
        );
        self.toggle()?;
        self.timer_driver
            .enable()
            .map_err(DigitalOutError::TimerDriverError)
    }

    /// Handles the diferent type of interrupts and reenabling the interrupt when necesary
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalOutError` if the operation fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPin`: If the pin level cannot be toggled.
    fn _update_interrupt(&mut self) -> Result<(), DigitalOutError> {
        let interrupt_update = InterruptUpdate::from_atomic_code(&self.interrupt_update_code);
        self.interrupt_update_code
            .store(InterruptUpdate::None.get_code(), Ordering::SeqCst);

        match interrupt_update {
            InterruptUpdate::Blink => self.toggle(),
            InterruptUpdate::None => Ok(()),
        }
    }
}

impl DigitalOut<'_> {
    /// Creates a new `DigitalOut` for a specified pin.
    ///
    /// # Arguments
    ///
    /// - `timer_driver`: A TimerDriver<'a> instance to manage the timing operations.
    /// - `per`: A Peripheral that can be transformed into an AnyIOPin.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `DigitalOut` instance, or a `DigitalOutError` if the initialization fails.
    ///
    /// # Errors
    ///
    /// - `DigitalOutError::InvalidPeripheral`: If the peripheral cannot be converted into an AnyIOPin.
    /// - `DigitalOutError::CannotSetPinAsOutput`: If the pin cannot be set as an output.
    pub(crate) fn new(
        timer_driver: TimerDriver,
        per: Peripheral,
    ) -> Result<DigitalOut, DigitalOutError> {
        Ok(DigitalOut {
            inner: SharableRef::new_sharable(_DigitalOut::new(timer_driver, per)?),
        })
    }
}

impl<'a> InterruptDriver<'a> for DigitalOut<'a> {
    /// Handles the diferent type of interrupts that, executing the user callback and reenabling the
    /// interrupt when necesary
    fn update_interrupt(&mut self) -> Result<(), Esp32FrameworkError> {
        self.inner.deref_mut()._update_interrupt()?;
        Ok(())
    }

    fn get_updater(&self) -> Box<dyn InterruptDriver<'a> + 'a> {
        Box::new(Self {
            inner: self.inner.clone(),
        })
    }
}

impl From<TimerDriverError> for DigitalOutError {
    fn from(value: TimerDriverError) -> Self {
        DigitalOutError::TimerDriverError(value)
    }
}
