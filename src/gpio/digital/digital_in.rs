use crate::{
    microcontroller_src::{
        interrupt_driver::InterruptDriver,
        peripherals::{Peripheral, PeripheralError},
    },
    utils::{
        auxiliary::{SharableRef, SharableRefExt},
        esp32_framework_error::Esp32FrameworkError,
        notification::Notifier,
        timer_driver::{TimerDriver, TimerDriverError},
    },
};
use esp_idf_svc::{
    hal::gpio::{InterruptType as SvcInterruptType, *},
    sys::{EspError, ESP_ERR_INVALID_STATE},
};
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

/// Enums the different interrupt types accepted when working with the digital in
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InterruptType {
    PosEdge,
    NegEdge,
    AnyEdgeNextEdgeIsPos,
    AnyEdgeNextEdgeIsNeg,
    LowLevel,
    HighLevel,
}

/// Driver for receiving digital inputs from a particular Pin
/// - `pin_driver`: An instance of `PinDriver` that implements AnyIOPin
/// - `timer_driver`: An instance of `TimerDriver`
/// - `interrupt_type`: An `InterruptType` to handle interrupts on the correct moment
/// - `interrupt_update_code`: `Arc<AtomicInterruptUpdateCode>` that indicates how to handle the interrupt
/// - `user_callback`: A closure to execute when the interrupt activates
/// - `debounce_ms`: An `Option` containing an u64 representing the debounce time in milliseconds
/// - `notifier`: An `Option<notifier>` in order to wake up the [crate::Microcontroller] after an interrupt
struct _DigitalIn<'a> {
    pin_driver: PinDriver<'a, AnyIOPin, Input>,
    timer_driver: TimerDriver<'a>,
    interrupt_type: Option<InterruptType>,
    interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
    user_callback: Box<dyn FnMut(Level)>,
    debounce_us: Option<u64>,
    notifier: Option<Notifier>,
}

/// Driver for receiving digital inputs from a particular Pin
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
    fn from_atomic_code(atomic_code: &Arc<AtomicInterruptUpdateCode>) -> Self {
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
    /// - `DigitalInError::CannotSetPullForPin`: If the pin driver is unable to support a setting of the pull
    fn new(
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
            debounce_us: None,
            user_callback: Box::new(|_| {}),
            notifier,
        };

        digital_in.set_pull(Pull::Down)?;
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

    /// Changes the interrupt type.
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
    /// - `DigitalInError::InvalidPin`: If the pin driver is unable to support a setting of an interrupt type
    pub fn change_interrupt_type(
        &mut self,
        interrupt_type: InterruptType,
    ) -> Result<(), DigitalInError> {
        self.interrupt_type = Some(interrupt_type);
        self.pin_driver
            .set_interrupt_type(interrupt_type.to_svc())
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
    /// Closure that can be called to start the timer,
    fn trigger_if_mantains_after(&mut self, time_micro: u64) -> impl FnMut() + Send + 'static {
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let after_timer_cljr = move || {
            interrupt_update_code_ref
                .store(InterruptUpdate::TimerReached.get_code(), Ordering::SeqCst);
        };

        self.timer_driver
            .interrupt_after(time_micro, after_timer_cljr);

        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        move || {
            interrupt_update_code_ref.store(
                InterruptUpdate::EnableTimerDriver.get_code(),
                Ordering::SeqCst,
            );
        }
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
    /// # Errors
    ///
    /// - `DigitalInError::InvalidPin`: If the pin driver is unable to support a setting of an interrupt type
    /// - `DigitalInError::StateAlreadySet`: If state was already set.
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
                        .map_err(DigitalInError::from_enable_disable_errors)?;
                };
            }
            None => unsafe {
                self.pin_driver
                    .subscribe(func)
                    .map_err(DigitalInError::from_enable_disable_errors)?;
            },
        };

        self.pin_driver
            .enable_interrupt()
            .map_err(DigitalInError::from_enable_disable_errors)
    }

    /// Sets a callback that sets an InterruptUpdate on the received interrupt type, which will then
    /// execute the user callback. If a debounce is set then the level must be mantained for the
    /// user callback to be executed.
    ///
    /// # Arguments
    ///
    /// - `user_callback`: A function provided by the user that is executed when the interrupt condition
    ///                    is met. This callback receives the current level of the pin.
    /// - `callback`: A function that is called when the interrupt is triggered.
    /// - `interrupt_type`: The InterruptType to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while setting up the interrupt.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::InvalidPin`: If the pin driver is unable to support a setting of an interrupt type.
    /// - `DigitalInError::StateAlreadySet`: If state was already set.
    fn _trigger_on_interrupt<G: FnMut() + Send + 'static, F: FnMut(Level) + 'static>(
        &mut self,
        user_callback: F,
        callback: G,
        interrupt_type: InterruptType,
    ) -> Result<(), DigitalInError> {
        self.change_interrupt_type(interrupt_type)?;
        self.user_callback = Box::new(user_callback);
        match self.debounce_us {
            Some(debounce_ms) => {
                let wrapper = self.trigger_if_mantains_after(debounce_ms);
                self.subscribe_trigger(wrapper)
            }
            None => self.subscribe_trigger(callback),
        }
    }

    /// Sets a callback that sets an InterruptUpdate on the received interrupt type, which will then
    /// execute the user callback. If a debounce is set then the level must be mantained for the
    /// user callback to be executed.
    ///
    /// Note: For the callback to be executed, the method [crate::Microcontroller::wait_for_updates] must
    /// be called periodicly, unless using an async aproach in which case [crate::Microcontroller::block_on]
    /// must be used.
    ///
    /// # Arguments
    ///
    /// - `user_callback`: A function provided by the user that is executed when the interrupt condition
    ///                    is met. This callback receives the current level of the pin.
    /// - `interrupt_type`: The `InterruptType` to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while setting up the interrupt.
    ///
    /// # Errors
    ///
    /// DigitalInError::InvalidPin: If the pin driver is unable to support a setting of an interrupt type.
    /// DigitalInError::StateAlreadySet: If state was already set.
    pub fn trigger_on_interrupt<F: FnMut(Level) + 'static>(
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
    /// Note: For the callback to be executed, the method [crate::Microcontroller::wait_for_updates] must
    /// be called periodicly, unless using an async aproach in which case [crate::Microcontroller::block_on]
    /// must be used.
    ///
    /// # Arguments
    ///
    /// - `amount_of_times`: The number of times the interrupt will trigger before unsubscribing.
    /// - `user_callback`: A function provided by the user that is executed when the interrupt condition
    ///                    is met. This callback receives the current level of the pin.
    /// - `interrupt_type`: The `InterruptType` to set for the pin.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while setting up the interrupt.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::InvalidPin`: If the pin driver is unable to support a setting of an interrupt type.
    /// - `DigitalInError::StateAlreadySet`: If state was already set.
    pub fn trigger_on_interrupt_first_n_times<F: FnMut(Level) + 'static>(
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
    /// change from the messurement before the debounce time, so the user callback is executed.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while processing the timer.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::NoInterruptTypeSet`: If the interrupt_type is None
    /// - `DigitalInError::InvalidPin`: If enabling of the interrupt fails
    /// - `DigitalInError::StateAlreadySet`: If the ISR service has not been initialized
    fn timer_reached(&mut self) -> Result<(), DigitalInError> {
        let level = self.get_interrupt_level()?;

        if self.pin_driver.get_level() == level {
            (self.user_callback)(level);
            self.swap_on_any_edge()?;
        }

        self.pin_driver
            .enable_interrupt()
            .map_err(DigitalInError::from_enable_disable_errors)
    }

    /// Change the InterruptType depending on the current InterruptType. This only apply for
    /// "AnyEdge" type of interrupts.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs while processing the timer.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::InvalidPin`: If the pin driver is unable to support a setting of an interrupt type.
    fn swap_on_any_edge(&mut self) -> Result<(), DigitalInError> {
        if let Some(interrupt_type) = self.interrupt_type {
            match interrupt_type {
                InterruptType::AnyEdgeNextEdgeIsPos => {
                    self.change_interrupt_type(InterruptType::AnyEdgeNextEdgeIsNeg)?
                }
                InterruptType::AnyEdgeNextEdgeIsNeg => {
                    self.change_interrupt_type(InterruptType::AnyEdgeNextEdgeIsPos)?
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Gets the level corresponding to the currently set `self.interrupt_type`.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if no interrupt has been set.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::NoInterruptTypeSet`: If the interrupt type can not be set.
    fn get_interrupt_level(&self) -> Result<Level, DigitalInError> {
        match self.interrupt_type {
            Some(interrupt_type) => Ok(interrupt_type.get_level()),
            None => Err(DigitalInError::NoInterruptTypeSet),
        }
    }

    /// Handles the diferent type of interrupts, executing the user callback and reenabling the
    /// interrupts when necesary.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or a `DigitalInError` if an error occurs during interrupt handling.
    ///
    /// # Errors
    ///
    /// - `DigitalInError::TimerDriverError`: If the enabling of the timer driver fails.
    /// - `DigitalInError::InvalidPin`: If enabling of the interrupt fails.
    /// - `DigitalInError::StateAlreadySet`: If the ISR service has not been initialized.
    fn _update_interrupt(&mut self) -> Result<(), DigitalInError> {
        let interrupt_update = InterruptUpdate::from_atomic_code(&self.interrupt_update_code);
        self.interrupt_update_code
            .store(InterruptUpdate::None.get_code(), Ordering::SeqCst);

        match interrupt_update {
            InterruptUpdate::ExecAndEnablePin => {
                let level = self.get_interrupt_level()?;
                (self.user_callback)(level);
                self.swap_on_any_edge()?;
                self.pin_driver
                    .enable_interrupt()
                    .map_err(DigitalInError::from_enable_disable_errors)
            }
            InterruptUpdate::EnableTimerDriver => self
                .timer_driver
                .enable()
                .map_err(DigitalInError::TimerDriverError),
            InterruptUpdate::TimerReached => self.timer_reached(),
            InterruptUpdate::ExecAndUnsubscribePin => {
                let level = self.get_interrupt_level()?;
                (self.user_callback)(level);
                self.pin_driver
                    .unsubscribe()
                    .map_err(DigitalInError::from_enable_disable_errors)
            }
            InterruptUpdate::None => Ok(()),
        }
    }

    /// Gets the current pin level.
    ///
    /// # Returns
    ///
    /// The current `Level` of the pin.
    pub fn get_level(&self) -> Level {
        self.pin_driver.get_level()
    }

    /// Verifies if the pin level is High.
    ///
    /// # Returns
    ///
    /// `true` if the pin level is `High`, otherwise `false`.
    pub fn is_high(&self) -> bool {
        self.pin_driver.get_level() == Level::High
    }

    /// Verifies if the pin level is Low.
    ///
    /// # Returns
    ///
    /// `true` if the pin level is `Low`, otherwise `false`.
    pub fn is_low(&self) -> bool {
        self.pin_driver.get_level() == Level::Low
    }

    /// Sets the debounce time to an amount of microseconds. This means that if an interrupt is set,
    /// then the level must be the same after the debounce time for the user callback to be executed.
    ///
    /// # Arguments
    ///
    /// - `time_micro`: The debounce time in microseconds.
    pub fn set_debounce(&mut self, time_micro: u64) {
        self.debounce_us = Some(time_micro)
    }
}

impl DigitalIn<'_> {
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
    /// - `DigitalInError::CannotSetPullForPin`: If the pin driver is unable to support a setting of the pull
    pub(crate) fn new(
        timer_driver: TimerDriver,
        per: Peripheral,
        notifier: Option<Notifier>,
    ) -> Result<DigitalIn, DigitalInError> {
        Ok(DigitalIn {
            inner: SharableRef::new_sharable(_DigitalIn::new(timer_driver, per, notifier)?),
        })
    }
}

impl<'a> InterruptDriver<'a> for DigitalIn<'a> {
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

impl From<TimerDriverError> for DigitalInError {
    fn from(value: TimerDriverError) -> Self {
        DigitalInError::TimerDriverError(value)
    }
}

/// Maps an error to a `DigitalInError` for enable disable errors
///
/// # Arguments
///
/// * `err` - The error to translate.
///
/// # Returns
///
/// A `DigitalInError` variant representing the translated error:
/// - `DigitalInError::StateAlreadySet`.
/// - `DigitalInError::InvalidPin`.
impl DigitalInError {
    fn from_enable_disable_errors(err: EspError) -> DigitalInError {
        match err.code() {
            ESP_ERR_INVALID_STATE => DigitalInError::StateAlreadySet,
            _ => DigitalInError::InvalidPin,
        }
    }
}

impl InterruptType {
    /// Converts the `Interrupt type`, into the corresponding `SvcInterruptType`.
    ///
    /// # Returns
    ///
    /// A `SvcInterruptType` instance.
    fn to_svc(self) -> SvcInterruptType {
        match self {
            InterruptType::PosEdge => SvcInterruptType::PosEdge,
            InterruptType::NegEdge => SvcInterruptType::NegEdge,
            InterruptType::AnyEdgeNextEdgeIsPos => SvcInterruptType::PosEdge,
            InterruptType::AnyEdgeNextEdgeIsNeg => SvcInterruptType::NegEdge,
            InterruptType::LowLevel => SvcInterruptType::LowLevel,
            InterruptType::HighLevel => SvcInterruptType::HighLevel,
        }
    }

    /// Gets the corresponding `Level` according to Self.
    ///
    /// # Returns
    ///
    /// A `Level` instance.
    fn get_level(&self) -> Level {
        match self {
            InterruptType::PosEdge => Level::High,
            InterruptType::NegEdge => Level::Low,
            InterruptType::AnyEdgeNextEdgeIsPos => Level::High,
            InterruptType::AnyEdgeNextEdgeIsNeg => Level::Low,
            InterruptType::LowLevel => Level::Low,
            InterruptType::HighLevel => Level::High,
        }
    }
}
