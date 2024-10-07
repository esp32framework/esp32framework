use crate::{
    microcontroller_src::{
        interrupt_driver::InterruptDriver,
        peripherals::{Peripheral, PeripheralError},
    },
    utils::{
        auxiliary::{SharableRef, SharableRefExt},
        error_text_parser::map_enable_disable_errors,
        esp32_framework_error::Esp32FrameworkError,
        notification::Notifier,
        timer_driver::{TimerDriver, TimerDriverError},
    },
};
pub use esp_idf_svc::hal::gpio::InterruptType;
use esp_idf_svc::hal::gpio::*;
use sharable_reference_macro::sharable_reference_wrapper;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};

type AtomicInterruptUpdateCode = AtomicU8;

/// Enums the different errors possible when working with the digital in
#[derive(Debug)]
pub enum DigitalInError {
    CannotSetDebounceOnAnyEdgeInterruptType,
    CannotSetPinAsInput,
    CannotSetPullForPin,
    InvalidPeripheral(PeripheralError),
    InvalidPin,
    NoInterruptTypeSet,
    StateAlreadySet,
    TimerDriverError(TimerDriverError),
}

/// Driver for receiving digital inputs from a particular Pin
/// - `pin_driver`: An instance of PinDriver that implements AnyIOPin
/// - `timer_driver`: An instance of TimerDriver
/// - `interrupt_type`: An InterruptType to handle interrupts on the correct moment
/// - `interrupt_update_code`: Arc<AtomicInterruptUpdateCode> that indicates how to handle the interrupt
/// - `user_callback`: A closure to execute when the interrupt activates
/// - `debounce_ms`: An Option containing an u64 representing the debounce time in milliseconds
/// - `notifier`: An Option<notifier> in order to wake up the [crate::Microcontroller] after an interrupt
struct _DigitalIn<'a> {
    pub pin_driver: PinDriver<'a, AnyIOPin, Input>,
    timer_driver: TimerDriver<'a>,
    interrupt_type: Option<InterruptType>,
    pub interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
    user_callback: Box<dyn FnMut()>,
    debounce_ms: Option<u64>,
    notifier: Option<Notifier>,
}

/// Driver for receiving digital inputs from a particular Pin
#[derive(Clone)]
pub struct DigitalIn<'a> {
    inner: SharableRef<_DigitalIn<'a>>,
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
enum InterruptUpdate {
    EnableTimerDriver,
    ExecAndEnablePin,
    ExecAndUnsubscribePin,
    None,
    TimerReached,
}

impl InterruptUpdate {
    /// Retrieves the interrupt code as a `u8`.
    ///
    /// # Returns
    ///
    /// A `u8` representing the interrupt code corresponding to the variant of `InterruptUpdate`.
    fn get_code(self) -> u8 {
        self as u8
    }

    /// Converts the interrupt code into an atomic version.
    ///
    /// # Returns
    ///
    /// An `AtomicInterruptUpdateCode` initialized with the current interrupt code.
    fn get_atomic_code(self) -> AtomicInterruptUpdateCode {
        AtomicInterruptUpdateCode::new(self.get_code())
    }

    /// Creates an `InterruptUpdate` variant from a given interrupt code.
    ///
    /// # Arguments
    ///
    /// - `code`: A `u8` representing the interrupt code.
    ///
    /// # Returns
    ///
    /// An `InterruptUpdate` variant corresponding to the provided code.
    ///
    /// # Example
    ///
    /// ```
    /// let interrupt = InterruptUpdate::from_code(1);
    /// assert_eq!(interrupt, InterruptUpdate::ExecAndEnablePin);
    /// ```
    fn from_code(code: u8) -> Self {
        match code {
            x if x == Self::ExecAndEnablePin.get_code() => Self::ExecAndEnablePin,
            x if x == Self::EnableTimerDriver.get_code() => Self::EnableTimerDriver,
            x if x == Self::TimerReached.get_code() => Self::TimerReached,
            x if x == Self::ExecAndUnsubscribePin.get_code() => Self::ExecAndUnsubscribePin,
            _ => Self::None,
        }
    }

    /// Converts an `AtomicInterruptUpdateCode` into an `InterruptUpdate` variant.
    ///
    /// # Arguments
    ///
    /// - `atomic_code`: An `Arc<AtomicInterruptUpdateCode>` containing the atomic interrupt code.
    ///
    /// # Returns
    ///
    /// An `InterruptUpdate` variant corresponding to the loaded atomic code.
    fn from_atomic_code(atomic_code: Arc<AtomicInterruptUpdateCode>) -> Self {
        InterruptUpdate::from_code(atomic_code.load(Ordering::Acquire))
    }
}

#[sharable_reference_wrapper]
impl<'a> _DigitalIn<'a> {
    /// Create a new DigitalIn for a Pin by default pull is set to Down.
    ///
    /// # Arguments
    ///
    /// - `timer_driver`: A TimerDriver that manages timing-related operations.
    /// - `per`: A Peripheral capable of transforming into an AnyIOPin.
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller] after an interrupt
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_DigitalIn` instance, or a `DigitalInError` if initialization fails.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::InvalidPeripheral`: If per parameter is not capable of transforming into an AnyIOPin, 
    ///   or pin has already been used for another driver.
    /// - `DigitalInError::CannotSetPinAsInput`: If the per parameter is not capable of soportin input
    ///
    /// # Panics
    ///
    /// When setting Down the pull fails
    pub fn new(
        timer_driver: TimerDriver<'a>,
        per: Peripheral,
        notifier: Option<Notifier>,
    ) -> Result<_DigitalIn<'a>, DigitalInError> {
        let gpio = per
            .into_any_io_pin()
            .map_err(DigitalInError::InvalidPeripheral)?;
        let pin_driver = PinDriver::input(gpio).map_err(|_| DigitalInError::CannotSetPinAsInput)?;

        let mut digital_in = _DigitalIn {
            pin_driver,
            timer_driver,
            interrupt_type: None,
            interrupt_update_code: Arc::from(InterruptUpdate::None.get_atomic_code()),
            debounce_ms: None,
            user_callback: Box::new(|| {}),
            notifier,
        };

        digital_in.set_pull(Pull::Down).unwrap();
        Ok(digital_in)
    }

    /// Set the pin Pull either to Pull Up or Down
    ///
    /// # Arguments
    ///
    /// - `pull_type`: The Pull type to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if the pull cannot be set.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::CannotSetPullForPin`: If the pin driver is unable to support a setting of the pull
    pub fn set_pull(&mut self, pull_type: Pull) -> Result<(), DigitalInError> {
        self.pin_driver
            .set_pull(pull_type)
            .map_err(|_| DigitalInError::CannotSetPullForPin)
    }

    /// Changes the interrupt type, fails if a debounce time is set and the interrupt type is AnyEdge
    ///
    /// # Arguments
    ///
    /// - `interrupt_type`: The InterruptType to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if the interrupt type cannot be set.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType`: If a debounce time is set and the interrupt type is `AnyEdge`.
    /// - `DigitalInError::InvalidPin`: If the pin driver is unable to support a setting of an interrupt type
    pub fn change_interrupt_type(
        &mut self,
        interrupt_type: InterruptType,
    ) -> Result<(), DigitalInError> {
        if let InterruptType::AnyEdge = interrupt_type {
            return Err(DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType);
        }
        self.interrupt_type = Some(interrupt_type);
        self.pin_driver
            .set_interrupt_type(interrupt_type)
            .map_err(|_| DigitalInError::InvalidPin)
    }

    /// After an interrupt, sets an interrupt that will trigger after an amount of microseconds. If the
    /// Level remains the same afterwards then the interrupt update is set to execute user callback
    ///
    /// # Arguments
    ///
    /// - `time_micro`: The time in microseconds after which the interrupt will trigger.
    ///
    /// # Returns
    ///
    /// A `Result` containing a closure that can be called to start the timer, or a `DigitalInError` if an error occurs.
    fn trigger_if_mantains_after(
        &mut self,
        time_micro: u64,
    ) -> Result<impl FnMut() + Send + 'static, DigitalInError> {
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let after_timer_cljr = move || {
            interrupt_update_code_ref
                .store(InterruptUpdate::TimerReached.get_code(), Ordering::SeqCst);
        };

        self.timer_driver
            .interrupt_after(time_micro, after_timer_cljr);

        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let start_timer_cljr = move || {
            interrupt_update_code_ref.store(
                InterruptUpdate::EnableTimerDriver.get_code(),
                Ordering::SeqCst,
            );
        };

        Ok(start_timer_cljr)
    }

    /// Subscribes the function to the pin driver interrupt and enables it
    ///
    /// # Arguments
    ///
    /// - `func`: The function to be executed when the interrupt is triggered.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if the subscription fails.
    ///
    /// # Panics
    ///
    /// When creating a new instance of a NonZeroU32 fails
    fn subscribe_trigger<F: FnMut() + Send + 'static>(
        &mut self,
        mut func: F,
    ) -> Result<(), DigitalInError> {
        match &self.notifier {
            Some(notifier) => {
                let notif = notifier.clone();
                let callback = move || {
                    func();
                    notif.notify();
                };
                unsafe {
                    self.pin_driver
                        .subscribe(callback)
                        .map_err(map_enable_disable_errors)?;
                };
            }
            None => unsafe {
                self.pin_driver
                    .subscribe(func)
                    .map_err(map_enable_disable_errors)?;
            },
        };

        self.pin_driver
            .enable_interrupt()
            .map_err(map_enable_disable_errors)
    }

    /// Sets a callback that sets an InterruptUpdate on the received interrupt type, which will then
    /// execute the user callback. If a debounce is set then the level must be mantained for the
    /// user callback to be executed.
    ///
    /// # Arguments
    ///
    /// - `user_callback`: A function provided by the user that is executed when the interrupt condition is met.
    /// - `callback`: A function that is called when the interrupt is triggered.
    /// - `interrupt_type`: The InterruptType to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while setting up the interrupt.
    pub fn _trigger_on_interrupt<G: FnMut() + Send + 'static, F: FnMut() + 'static>(
        &mut self,
        user_callback: F,
        callback: G,
        interrupt_type: InterruptType,
    ) -> Result<(), DigitalInError> {
        self.change_interrupt_type(interrupt_type)?;
        self.user_callback = Box::new(user_callback);
        match self.debounce_ms {
            Some(debounce_ms) => {
                let wrapper = self.trigger_if_mantains_after(debounce_ms)?;
                self.subscribe_trigger(wrapper)
            }
            None => self.subscribe_trigger(callback),
        }
    }

    /// Sets a callback that sets an InterruptUpdate on the received interrupt type, which will then
    /// execute the user callback. If a debounce is set then the level must be mantained for the
    /// user callback to be executed.
    ///
    /// # Arguments
    ///
    /// - `user_callback`: A function that is executed when the interrupt condition is met.
    /// - `interrupt_type`: The `InterruptType` to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while setting up the interrupt.
    pub fn trigger_on_interrupt<F: FnMut() + 'static>(
        &mut self,
        user_callback: F,
        interrupt_type: InterruptType,
    ) -> Result<(), DigitalInError> {
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            interrupt_update_code_ref.store(
                InterruptUpdate::ExecAndEnablePin.get_code(),
                Ordering::SeqCst,
            );
        };
        self._trigger_on_interrupt(user_callback, callback, interrupt_type)
    }

    /// Sets a callback to be triggered only n times before unsubscribing the interrupt.
    ///
    /// # Arguments
    ///
    /// - `amount_of_times`: The number of times the interrupt will trigger before unsubscribing.
    /// - `user_callback`: A function that is executed when the interrupt condition is met.
    /// - `interrupt_type`: The `InterruptType` to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while setting up the interrupt.
    pub fn trigger_on_interrupt_first_n_times<F: FnMut() + 'static>(
        &mut self,
        amount_of_times: usize,
        user_callback: F,
        interrupt_type: InterruptType,
    ) -> Result<(), DigitalInError> {
        if amount_of_times == 0 {
            return Ok(());
        }

        let mut amount_of_times = amount_of_times;
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            amount_of_times -= 1;
            if amount_of_times == 0 {
                interrupt_update_code_ref.store(
                    InterruptUpdate::ExecAndUnsubscribePin.get_code(),
                    Ordering::SeqCst,
                );
            } else {
                interrupt_update_code_ref.store(
                    InterruptUpdate::ExecAndEnablePin.get_code(),
                    Ordering::SeqCst,
                );
            }
        };
        self._trigger_on_interrupt(user_callback, callback, interrupt_type)
    }

    /// Checks if the level corresponds to the set interrupt type. If it does it means the level didnt
    /// change from the messurement before the debounce time, so the user callback is executed
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while processing the timer.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType`: If the interrupt_type is AnyEdge
    /// - `DigitalInError::NoInterruptTypeSet`: If the interrupt_type is None
    /// - `DigitalInError::InvalidPin`: If enabling of the interrupt fails
    /// - `DigitalInError::StateAlreadySet`: If the ISR service has not been initialized
    fn timer_reached(&mut self) -> Result<(), DigitalInError> {
        let level = match self.interrupt_type {
            Some(InterruptType::PosEdge) => Level::High,
            Some(InterruptType::NegEdge) => Level::Low,
            Some(InterruptType::AnyEdge) => {
                Err(DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType)?
            }
            Some(InterruptType::LowLevel) => Level::Low,
            Some(InterruptType::HighLevel) => Level::High,
            None => Err(DigitalInError::NoInterruptTypeSet)?,
        };

        if self.pin_driver.get_level() == level {
            (self.user_callback)();
        }

        self.pin_driver
            .enable_interrupt()
            .map_err(map_enable_disable_errors)
    }

    /// Handles the diferent type of interrupts that, executing the user callback and reenabling the
    /// interrupt when necesary
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs during interrupt handling.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::TimerDriverError`: If the enabling of the timer driver fails
    /// - `DigitalInError::InvalidPin`: If enabling of the interrupt fails
    /// - `DigitalInError::StateAlreadySet`: If the ISR service has not been initialized
    fn _update_interrupt(&mut self) -> Result<(), DigitalInError> {
        let interrupt_update =
            InterruptUpdate::from_atomic_code(self.interrupt_update_code.clone());
        self.interrupt_update_code
            .store(InterruptUpdate::None.get_code(), Ordering::SeqCst);

        match interrupt_update {
            InterruptUpdate::ExecAndEnablePin => {
                (self.user_callback)();
                self.pin_driver
                    .enable_interrupt()
                    .map_err(map_enable_disable_errors)
            }
            InterruptUpdate::EnableTimerDriver => self
                .timer_driver
                .enable()
                .map_err(DigitalInError::TimerDriverError),
            InterruptUpdate::TimerReached => self.timer_reached(),
            InterruptUpdate::ExecAndUnsubscribePin => {
                (self.user_callback)();
                self.pin_driver
                    .unsubscribe()
                    .map_err(map_enable_disable_errors)
            }
            InterruptUpdate::None => Ok(()),
        }
    }

    /// Gets the current pin level
    ///
    /// # Returns
    ///
    /// The current `Level` of the pin.
    pub fn get_level(&self) -> Level {
        self.pin_driver.get_level()
    }

    /// Verifies if the pin level is High
    ///
    /// # Returns
    ///
    /// `true` if the pin level is `High`, otherwise `false`.
    pub fn is_high(&self) -> bool {
        self.pin_driver.get_level() == Level::High
    }

    /// Verifies if the pin level is Low
    ///
    /// # Returns
    ///
    /// `true` if the pin level is `Low`, otherwise `false`.
    pub fn is_low(&self) -> bool {
        self.pin_driver.get_level() == Level::Low
    }

    /// Sets the debounce time to an amount of microseconds. This means that if an interrupt is set,
    /// then the level must be the same after the debounce time for the user callback to be executed.
    /// Debounce time does not work with InterruptType::AnyEdge, an error will be returned
    ///
    /// # Arguments
    ///
    /// - `time_micro`: The debounce time in microseconds.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if the debounce time cannot be set due to the interrupt type.
    pub fn set_debounce(&mut self, time_micro: u64) -> Result<(), DigitalInError> {
        match self.interrupt_type {
            Some(InterruptType::AnyEdge) => {
                Err(DigitalInError::CannotSetDebounceOnAnyEdgeInterruptType)?
            }
            _ => self.debounce_ms = Some(time_micro),
        }
        Ok(())
    }
}

impl<'a> DigitalIn<'a> {
    /// Create a new DigitalIn for a Pin by default pull is set to Down.
    ///
    /// # Arguments
    ///
    /// - `timer_driver`: A TimerDriver that manages timing-related operations.
    /// - `per`: A Peripheral capable of transforming into an AnyIOPin.
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller] after an interrupt
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_DigitalIn` instance, or a `DigitalInError` if initialization fails.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::InvalidPeripheral`: If per parameter is not capable of transforming into an AnyIOPin,
    ///   or pin has already been used for another driver.
    /// - `DigitalInError::CannotSetPinAsInput`: If the per parameter is not capable of soportin input
    ///
    /// # Panics
    ///
    /// When setting Down the pull fails
    pub fn new(
        timer_driver: TimerDriver,
        per: Peripheral,
        notifier: Option<Notifier>,
    ) -> Result<DigitalIn, DigitalInError> {
        Ok(DigitalIn {
            inner: SharableRef::new_sharable(_DigitalIn::new(timer_driver, per, notifier)?),
        })
    }
}

#[sharable_reference_wrapper]
impl<'a> InterruptDriver for _DigitalIn<'a> {
    /// Handles the diferent type of interrupts that, executing the user callback and reenabling the
    /// interrupt when necesary
    fn update_interrupt(&mut self) -> Result<(), Esp32FrameworkError> {
        self._update_interrupt()
            .map_err(Esp32FrameworkError::DigitalIn)
    }
}

impl From<TimerDriverError> for DigitalInError {
    fn from(value: TimerDriverError) -> Self {
        DigitalInError::TimerDriverError(value)
    }
}
