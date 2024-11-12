use crate::{
    microcontroller_src::{
        interrupt_driver::InterruptDriver,
        peripherals::{Peripheral, PeripheralError},
    },
    utils::{
        auxiliary::{SharableRef, SharableRefExt},
        esp32_framework_error::Esp32FrameworkError,
        timer_driver::{TimerDriver, TimerDriverError},
    },
};
use esp_idf_svc::{
    hal::{ledc::*, peripheral, prelude::*},
    sys::ESP_FAIL,
};
use sharable_reference_macro::sharable_reference_wrapper;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
};

/// Enums the different errors possible when working with the analog out
#[derive(Debug)]
pub enum AnalogOutError {
    ErrorSettingOutput,
    InvalidArg,
    InvalidPeripheral(PeripheralError),
    InvalidFrequencyOrDuty,
    TimerDriverError(TimerDriverError),
    TooManyPWMOutputs,
}

/// Enums the possible Duty Policies for the driver
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExtremeDutyPolicy {
    BounceBack,
    None,
    Reset,
}

/// Enums Change Type of the drivers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FixedChangeType {
    Decrease(ExtremeDutyPolicy),
    Increase(ExtremeDutyPolicy),
    None,
}

/// Driver to handle an analog output for a particular pin
/// - `driver`: A `LedCDriver` instance that handles the PWM output signals
/// - `timer_driver`: A `TimerDriver` instance
/// - `duty`: The level of output duty
/// - `change_duty_update`: ChangeDutyUpdate that indicates if a change on the duty is needed
/// - `fixed_change_increasing`: `Arc<AtomicBool>` that indicates if a fixed change on the duty is needed
/// - `fixed_change_type`: An instance of `FixedChangeType` that indicates the type of duty change
/// - `amount_of_cycles`: An Option containing an `u32` thath indicates the amount of desired cycles
struct _AnalogOut<'a> {
    driver: LedcDriver<'a>,
    timer_driver: TimerDriver<'a>,
    duty: Arc<AtomicU32>,
    change_duty_update: ChangeDutyUpdate,
    fixed_change_increasing: Arc<AtomicBool>,
    fixed_change_type: FixedChangeType,
    amount_of_cycles: Option<u32>,
}

/// Driver to handle an analog output for a particular pin.
pub struct AnalogOut<'a> {
    inner: SharableRef<_AnalogOut<'a>>,
}

/// Wrapper for simple use of an `Arc<AtomicBool>`
/// in the context of the changinf of the drivers duty
#[derive(Clone, Debug)]
struct ChangeDutyUpdate {
    change: Arc<AtomicBool>,
}

impl FixedChangeType {
    /// Indicates if the starting of the cycle is from the starting point or not
    ///
    /// # Returns
    ///
    /// A bool. True if the cycle needs to start from the starting point, False if the
    /// cycle needs to start from the end point.
    fn increasing_starting_direction(&self) -> bool {
        matches!(self, FixedChangeType::Increase(_policy))
    }
}

impl ChangeDutyUpdate {
    /// Creates a new ChangeDutyUpdate instance
    ///
    /// # Returns
    ///
    /// The new ChangeDutyUpdate instance
    fn new() -> Self {
        ChangeDutyUpdate {
            change: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Changes the state to True
    fn change_duty(&mut self) {
        self.change.store(true, Ordering::Relaxed)
    }

    /// Changes the state to False
    ///
    /// # Returns
    ///
    /// A bool representing the previous state
    fn handle_change_duty(&mut self) -> bool {
        let change_duty = self.change.load(Ordering::Relaxed);
        self.change.store(false, Ordering::Relaxed);
        change_duty
    }
}

#[sharable_reference_wrapper]
impl<'a> _AnalogOut<'a> {
    /// Creates a new _AnalogOut from a pin number, frequency and resolution.
    ///
    /// # Arguments
    ///
    /// - `peripheral_channel`: A `Peripheral` instance of type `PWMChannel`
    /// - `timer`: A `Peripheral` instance of type `PWMTimer`
    /// - `gpio_pin`: A `Peripheral` capable of being transformed into an `AnyIOPin`
    /// - `timer_driver`: An instance of a TimerDriver
    /// - `freq_hz`: An `u32` representing the frequency in hertz desired for the configuration of the `PWMTimer`
    /// - `resolution`: An `u32` that represents the amount of bits in the desired output resolution. If 0 its set to 1 bit, >= 14
    ///     14 bits of resolution are set  
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_AnalogOut` instance, or an `AnalogOutError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    fn new(
        peripheral_channel: Peripheral,
        timer: Peripheral,
        gpio_pin: Peripheral,
        timer_driver: TimerDriver<'a>,
        freq_hz: u32,
        resolution: u32,
    ) -> Result<_AnalogOut<'a>, AnalogOutError> {
        let resolution = _AnalogOut::create_resolution(resolution);
        let config = &config::TimerConfig::new()
            .frequency(freq_hz.Hz())
            .resolution(resolution);
        _AnalogOut::_new(peripheral_channel, timer, gpio_pin, timer_driver, config)
    }

    /// Creates a new `_AnalogOut` for a specific pin with a given configuration of frecuency and resolution.
    ///
    /// # Arguments
    ///
    /// - `peripheral_channel`: A `Peripheral` instance of type `PWMChannel`
    /// - `timer`: A `Peripheral` instance of type `PWMTimer`
    /// - `gpio_pin`: A `Peripheral` capable of being transformed into an `AnyIOPin`
    /// - `timer_driver`: An instance of a `TimerDriver`
    /// - `config`: An instance of `TimerConfig` with the frequency and resolution already set
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_AnalogOut` instance, or an `AnalogOutError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    fn _new(
        peripheral_channel: Peripheral,
        timer: Peripheral,
        gpio_pin: Peripheral,
        timer_driver: TimerDriver<'a>,
        config: &config::TimerConfig,
    ) -> Result<_AnalogOut<'a>, AnalogOutError> {
        let pwm_driver = match timer {
            Peripheral::PWMTimer(0) => _AnalogOut::create_pwm_driver(
                peripheral_channel,
                unsafe { TIMER0::new() },
                gpio_pin,
                config,
            ),
            Peripheral::PWMTimer(1) => _AnalogOut::create_pwm_driver(
                peripheral_channel,
                unsafe { TIMER1::new() },
                gpio_pin,
                config,
            ),
            Peripheral::PWMTimer(2) => _AnalogOut::create_pwm_driver(
                peripheral_channel,
                unsafe { TIMER2::new() },
                gpio_pin,
                config,
            ),
            Peripheral::PWMTimer(3) => _AnalogOut::create_pwm_driver(
                peripheral_channel,
                unsafe { TIMER3::new() },
                gpio_pin,
                config,
            ),
            Peripheral::None => Err(AnalogOutError::InvalidPeripheral(
                PeripheralError::AlreadyTaken,
            )),
            _ => Err(AnalogOutError::InvalidPeripheral(
                PeripheralError::NotAPwmTimer,
            ))?,
        }?;

        Ok(_AnalogOut {
            driver: pwm_driver,
            timer_driver,
            duty: Arc::new(AtomicU32::new(0)),
            change_duty_update: ChangeDutyUpdate::new(),
            fixed_change_increasing: Arc::new(AtomicBool::new(false)),
            fixed_change_type: FixedChangeType::None,
            amount_of_cycles: None,
        })
    }

    /// Creates a new _AnalogOut with a default frecuency of 1000Hz and a resolution of 8bits.
    ///
    /// # Arguments
    ///
    /// - `peripheral_channel`: A `Peripheral` instance of type PWMChannel
    /// - `timer`: A `Peripheral` instance of type PWMTimer
    /// - `gpio_pin`: A `Peripheral` capable of being transformed into an AnyIOPin
    /// - `timer_driver`: An instance of a TimerDriver
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_AnalogOut` instance, or an `AnalogOutError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    fn default(
        peripheral_channel: Peripheral,
        timer: Peripheral,
        gpio_pin: Peripheral,
        timer_driver: TimerDriver<'a>,
    ) -> Result<_AnalogOut<'a>, AnalogOutError> {
        _AnalogOut::_new(
            peripheral_channel,
            timer,
            gpio_pin,
            timer_driver,
            &config::TimerConfig::new(),
        )
    }

    /// Creates a new Resolution from a `u32` value.
    ///
    /// # Arguments
    ///
    /// - `resolution`: An `u32` representing the desired resolution. Accepted values go from 0 to 13
    ///
    /// # Returns
    ///
    /// A Resolution instance
    fn create_resolution(resolution: u32) -> Resolution {
        match resolution {
            0 => Resolution::Bits1,
            1 => Resolution::Bits1,
            2 => Resolution::Bits2,
            3 => Resolution::Bits3,
            4 => Resolution::Bits4,
            5 => Resolution::Bits5,
            6 => Resolution::Bits6,
            7 => Resolution::Bits7,
            8 => Resolution::Bits8,
            9 => Resolution::Bits9,
            10 => Resolution::Bits10,
            11 => Resolution::Bits11,
            12 => Resolution::Bits12,
            13 => Resolution::Bits13,
            _ => Resolution::Bits14,
        }
    }

    /// Creates a new LedcDriver from a Peripheral::PWMTimer
    ///
    /// # Arguments
    ///
    /// - `peripheral_channel`: A `Peripheral` of type `PWMChannel`
    /// - `timer`: An esp_idf_svc::peripheral::Peripheral
    /// - `gpio_pin`: A `Peripheral` capable of being transformed into an AnyIOPin
    /// - `config`: A TimerConfig
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `LedcDriver` instance, or an `AnalogOutError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the choosen frequency and duty are incompatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not of the correct type
    fn create_pwm_driver<L: 'a + LedcTimer<SpeedMode = LowSpeed>>(
        peripheral_channel: Peripheral,
        timer: impl peripheral::Peripheral<P = L> + 'a,
        gpio_pin: Peripheral,
        config: &config::TimerConfig,
    ) -> Result<LedcDriver<'a>, AnalogOutError> {
        let ledc_timer_driver =
            LedcTimerDriver::new(timer, config).map_err(|error| match error.code() {
                ESP_FAIL => AnalogOutError::InvalidFrequencyOrDuty,
                _ => AnalogOutError::InvalidArg,
            })?;

        let gpio = gpio_pin
            .into_any_io_pin()
            .map_err(AnalogOutError::InvalidPeripheral)?;

        match peripheral_channel {
            Peripheral::PWMChannel(0) => {
                LedcDriver::new(unsafe { CHANNEL0::new() }, ledc_timer_driver, gpio)
            }
            Peripheral::PWMChannel(1) => {
                LedcDriver::new(unsafe { CHANNEL1::new() }, ledc_timer_driver, gpio)
            }
            Peripheral::PWMChannel(2) => {
                LedcDriver::new(unsafe { CHANNEL2::new() }, ledc_timer_driver, gpio)
            }
            Peripheral::PWMChannel(3) => {
                LedcDriver::new(unsafe { CHANNEL3::new() }, ledc_timer_driver, gpio)
            }
            Peripheral::None => {
                return Err(AnalogOutError::InvalidPeripheral(
                    PeripheralError::AlreadyTaken,
                ))
            }
            _ => {
                return Err(AnalogOutError::InvalidPeripheral(
                    PeripheralError::NotAPwmChannel,
                ))
            }
        }
        .map_err(|_| AnalogOutError::InvalidArg)
    }

    /// Changes the output signal to be at it maximun by calling #[Self::set_high_level_output_ratio]
    /// with 1.0 as the high_ratio parameter.
    pub fn set_high(&mut self)-> Result<(), AnalogOutError>{
        self.set_high_level_output_ratio(1.0)
    }
    
    /// Changes the output signal to be at it minimum by calling #[Self::set_high_level_output_ratio]
    /// with 0.0 as the high_ratio parameter.
    pub fn set_low(&mut self)-> Result<(), AnalogOutError>{
        self.set_high_level_output_ratio(0.0)
    }

    /// Changes the intensity of the signal using the High-Low level ratio
    /// If the driver has been set to increase or decrease automaticly then calling this function
    /// will stop this behaviour.
    ///
    /// # Arguments
    ///
    /// - `high_ratio`: An `f32` representinf the desired high level ratio
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the set operation completed successfully, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::ErrorSettingOutput`: If the set operation fails
    /// - `TimerDriverError`: If an error occurs while removing the automatic change of dutty cycle
    pub fn set_high_level_output_ratio(&mut self, high_ratio: f32) -> Result<(), AnalogOutError> {
        let duty: u32 = duty_from_high_ratio(self.driver.get_max_duty(), high_ratio);
        self.fixed_change_type = FixedChangeType::None;
        self.timer_driver.remove_interrupt()?;
        self.duty.store(duty, Ordering::SeqCst);
        self.driver
            .set_duty(duty)
            .map_err(|_| AnalogOutError::ErrorSettingOutput)
    }

    /// Creates the proper callback and subscribes it to the TimerDriver
    ///
    /// # Arguments
    ///
    /// - `fixed_change_type`: A `FixedChangeType` enum that defines whether the duty cycle should increase or decrease.
    /// - `increase_after_miliseconds`: An `u64` representing the time interval (in milliseconds) after which the duty cycle should change.
    /// - `increase_by_ratio`: A `f32` representing the ratio by which the duty cycle should change.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    fn start_changing_by_fixed_amount(
        &mut self,
        fixed_change_type: FixedChangeType,
        increase_after_miliseconds: u64,
        increase_by_ratio: f32,
        starting_high_ratio: f32,
    ) -> Result<(), AnalogOutError> {
        let mut change_duty_update_ref = self.change_duty_update.clone();
        let duty_ref = self.duty.clone();
        let increase_direction_ref = self.fixed_change_increasing.clone();
        self.fixed_change_increasing.store(
            fixed_change_type.increasing_starting_direction(),
            Ordering::SeqCst,
        );
        let max_duty = self.driver.get_max_duty();

        let starting_duty = duty_from_high_ratio(max_duty, starting_high_ratio);
        duty_ref.store(starting_duty, Ordering::SeqCst);

        let callback = move || {
            let duty_step = duty_from_high_ratio(max_duty, increase_by_ratio).max(1);
            let new_duty = if increase_direction_ref.load(Ordering::Acquire) {
                (duty_ref.load(Ordering::Acquire) + duty_step).min(max_duty)
            } else {
                let prev_dutty = duty_ref.load(Ordering::Acquire);
                prev_dutty - prev_dutty.min(duty_step)
            };
            duty_ref.store(new_duty, Ordering::SeqCst);

            change_duty_update_ref.change_duty();
        };

        self.timer_driver.interrupt_after_n_times(
            increase_after_miliseconds * 1000,
            None,
            true,
            callback,
        );
        self.timer_driver
            .enable()
            .map_err(AnalogOutError::TimerDriverError)?;
        self.fixed_change_type = fixed_change_type;
        Ok(())
    }

    /// Sets the FixedChangeType to Increase. Stops when maximum ratio is reached.
    ///
    /// # Arguments
    ///
    /// - `increase_after_miliseconds`: An `u64` representing the time interval (in milliseconds) after which the duty cycle should increase.
    /// - `increase_by_ratio`: A `f32` representing the ratio by which the duty cycle should increase.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    pub fn start_increasing(
        &mut self,
        increase_after_miliseconds: u64,
        increase_by_ratio: f32,
        starting_high_ratio: f32,
    ) -> Result<(), AnalogOutError> {
        self.start_changing_by_fixed_amount(
            FixedChangeType::Increase(ExtremeDutyPolicy::None),
            increase_after_miliseconds,
            increase_by_ratio,
            starting_high_ratio,
        )
    }

    /// Sets the FixedChangeType to Decrease. Stops when minimum ratio is reached.
    ///
    /// # Arguments
    ///
    /// - `increase_after_miliseconds`: An `u64` representing the time interval (in milliseconds) after which the duty cycle should decrease.
    /// - `decrease_by_ratio`: A `f32` representing the ratio by which the duty cycle should decrease.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    pub fn start_decreasing(
        &mut self,
        increase_after_miliseconds: u64,
        decrease_by_ratio: f32,
        starting_high_ratio: f32,
    ) -> Result<(), AnalogOutError> {
        self.start_changing_by_fixed_amount(
            FixedChangeType::Decrease(ExtremeDutyPolicy::None),
            increase_after_miliseconds,
            decrease_by_ratio,
            starting_high_ratio,
        )
    }

    /// Increases the PWM signal ratio by `increase_by_ratio`, starting from `starting_high_ratio` value until it reaches the maximum ratio possible.
    /// Once the maximum is reached, it bounces back and starts to decrease until the minimum value is reached. Direction changes `amount_of_bounces` times
    /// unless that parameter is set to `None`, meaning it will do it indefinitely.
    ///
    /// # Arguments
    ///
    /// - `increase_after_miliseconds`: An `u64` representing the time interval (in milliseconds) after which the duty cycle should increase.
    /// - `increase_by_ratio`: A `f32` representing the ratio by which the duty cycle should increase.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    /// - `amount_of_bounces`: An `Option<u32>` representing the number of bounce-backs before stopping, or None for continuous bouncing.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    pub fn start_increasing_bounce_back(
        &mut self,
        increase_after_miliseconds: u64,
        increase_by_ratio: f32,
        starting_high_ratio: f32,
        amount_of_bounces: Option<u32>,
    ) -> Result<(), AnalogOutError> {
        self.amount_of_cycles = amount_of_bounces;
        self.start_changing_by_fixed_amount(
            FixedChangeType::Increase(ExtremeDutyPolicy::BounceBack),
            increase_after_miliseconds,
            increase_by_ratio,
            starting_high_ratio,
        )
    }

    /// Decreases the PWM signal ratio by 'decrease_by_ratio', starting from 'starting_high_ratio' value until it reaches the minimum ratio possible.
    /// Once the minimum is reached, it bounces back and starts to increase until the maximum value is reached. Direction changes 'amount_of_bounces' times
    /// unless that parameter is set to None, meaning it will do it indefinitely.
    ///
    /// # Arguments
    ///
    /// - `increase_after_miliseconds`: An `u64` representing the time interval (in milliseconds) after which the duty cycle should decrease.
    /// - `decrease_by_ratio`: A `f32` representing the ratio by which the duty cycle should decrease.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    /// - `amount_of_bounces`: An `Option<u32>` representing the number of bounce-backs before stopping, or None for continuous bouncing.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    pub fn start_decreasing_bounce_back(
        &mut self,
        increase_after_miliseconds: u64,
        decrease_by_ratio: f32,
        starting_high_ratio: f32,
        amount_of_bounces: Option<u32>,
    ) -> Result<(), AnalogOutError> {
        self.amount_of_cycles = amount_of_bounces;
        self.start_changing_by_fixed_amount(
            FixedChangeType::Decrease(ExtremeDutyPolicy::BounceBack),
            increase_after_miliseconds,
            decrease_by_ratio,
            starting_high_ratio,
        )
    }

    /// Increses the PWM signal ratio by `increase_by_ratio`, starting from `starting_high_ratio` value until it reaches the maximum ratio possible.
    /// Once the maximum is reached, it goes back to the `starting_high_ratio` and starts to increase once again. This is done `amount_of_resets` times
    /// unless that parameter is set to None, meaning it will do it indefinitely.
    ///  
    /// # Arguments
    ///
    /// - `increase_after_miliseconds`: A `u64` representing the time interval (in milliseconds) after which the duty cycle should increase.
    /// - `increase_by_ratio`: A `f32` representing the ratio by which the duty cycle should increase.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    /// - `amount_of_resets`: An `Option<u32>` representing the number of resets before stopping, or None for continuous resetting.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    pub fn start_increasing_reset(
        &mut self,
        increase_after_miliseconds: u64,
        increase_by_ratio: f32,
        starting_high_ratio: f32,
        amount_of_resets: Option<u32>,
    ) -> Result<(), AnalogOutError> {
        self.amount_of_cycles = amount_of_resets;
        self.start_changing_by_fixed_amount(
            FixedChangeType::Increase(ExtremeDutyPolicy::Reset),
            increase_after_miliseconds,
            increase_by_ratio,
            starting_high_ratio,
        )
    }

    /// Decreases the PWM signal ratio by 'decrease_by_ratio', starting from 'starting_high_ratio' value until it reaches the minimum ratio possible.
    /// Once the minimum is reached, it goes back to the 'starting_high_ratio' and starts to increase once again. This is done 'amount_of_resets' times
    /// unless that parameter is set to None, meaning it will do it indefinitely.
    ///
    /// # Arguments
    ///
    /// - `increase_after_miliseconds`: A `u64` representing the time interval (in milliseconds) after which the duty cycle should decrease.
    /// - `decrease_by_ratio`: A `f32` representing the ratio by which the duty cycle should decrease.
    /// - `starting_high_ratio`: A `f32` representing the initial high ratio for the duty cycle.
    /// - `amount_of_resets`: An `Option<u32>` representing the number of resets before stopping, or `None` for continuous resetting.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::TimerDriverError`: If the timer driver cannot be enabled.
    pub fn start_decreasing_reset(
        &mut self,
        increase_after_miliseconds: u64,
        decrease_by_ratio: f32,
        starting_high_ratio: f32,
        amount_of_resets: Option<u32>,
    ) -> Result<(), AnalogOutError> {
        self.amount_of_cycles = amount_of_resets;
        self.start_changing_by_fixed_amount(
            FixedChangeType::Decrease(ExtremeDutyPolicy::Reset),
            increase_after_miliseconds,
            decrease_by_ratio,
            starting_high_ratio,
        )
    }

    /// Changes the direction to 'increasing' if the direction is set to 'decreasing' and
    /// vice versa.
    fn turn_around(&mut self) {
        let previouse_direction = self.fixed_change_increasing.load(Ordering::Acquire);
        self.fixed_change_increasing
            .store(!previouse_direction, Ordering::SeqCst)
    }

    /// Amount of cycles can be a None or a Some(bounces). None means the turn around will be done indefinetly.
    /// Otherwise, the turn around will be done until the 'bounces' value becomes 0. Returns false if all the cycles
    /// were completed.
    ///
    /// # Returns
    ///
    /// A bool. True means it should turn around. False means it shouldn't
    fn attempt_turn_around(&mut self) -> bool {
        match self.amount_of_cycles {
            Some(bounces) => {
                if bounces > 0 {
                    self.turn_around();
                    self.amount_of_cycles.replace(bounces - 1);
                } else {
                    return false;
                }
            }
            None => self.turn_around(),
        }
        true
    }

    /// If direction 'increasing', the duty is set to 0. Otherwise, is set to the maximum duty possible
    fn reset(&mut self) {
        let increasing_direction = self.fixed_change_increasing.load(Ordering::Acquire);
        if increasing_direction {
            self.duty.store(0, Ordering::SeqCst)
        } else {
            self.duty
                .store(self.driver.get_max_duty(), Ordering::SeqCst)
        }
    }

    /// Amount of cycles can be a None or a Some(resets). None means the reset will be done indefinetly.
    /// Otherwise, the reset will be done until the 'resets' value becomes 0. Returns false if all the cycles
    /// were completed.
    ///
    /// # Returns
    ///
    /// A bool. True means it should reset. False means it shouldn't
    fn attempt_reset(&mut self) -> bool {
        match self.amount_of_cycles {
            Some(resets) => {
                if resets > 0 {
                    self.reset();
                    self.amount_of_cycles.replace(resets - 1);
                } else {
                    return false;
                }
            }
            None => self.reset(),
        }
        true
    }

    /// Handler for InterruptUpdate::ChangeDuty, depending on the ExtremeDutyPolicy
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation is successful, or an `AnalogOutError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::ErrorSettingOutput`: If setting the duty value fails
    /// - `AnalogOutError::TimerDriverError`: If removing the timer interrupt fails
    fn change_duty_on_cycle(&mut self) -> Result<(), AnalogOutError> {
        let duty = self.duty.load(Ordering::Acquire);
        let prev_duty = self.driver.get_duty();
        let mut stay_subscribed = true;

        if prev_duty == duty {
            stay_subscribed = match self.fixed_change_type {
                FixedChangeType::Increase(ExtremeDutyPolicy::BounceBack) => {
                    self.attempt_turn_around()
                }
                FixedChangeType::Decrease(ExtremeDutyPolicy::BounceBack) => {
                    self.attempt_turn_around()
                }
                FixedChangeType::Increase(ExtremeDutyPolicy::Reset) => self.attempt_reset(),
                FixedChangeType::Decrease(ExtremeDutyPolicy::Reset) => self.attempt_reset(),
                FixedChangeType::Increase(ExtremeDutyPolicy::None) => {
                    self.driver.get_duty() < self.driver.get_max_duty()
                }
                FixedChangeType::Decrease(ExtremeDutyPolicy::None) => self.driver.get_duty() > 0,
                _ => false,
            }
        }

        self.driver
            .set_duty(duty)
            .map_err(|_| AnalogOutError::ErrorSettingOutput)?;
        if !stay_subscribed {
            self.fixed_change_type = FixedChangeType::None;
            self.timer_driver
                .remove_interrupt()
                .map_err(AnalogOutError::TimerDriverError)?;
        }
        Ok(())
    }

    /// Handles the diferent type of interrupts.
    ///
    /// Returns a Result containing an AnalogOutError if an error ocurred.
    ///
    /// # Errors:
    /// - AnalogOutError::ErrorSettingOutput: In case of channel not initialized, parameter error or ESP_FAIL of fade function.
    /// - AnalogOutError::TimerDriverError: In case of failure removing the interrupt.
    fn _update_interrupt(&mut self) -> Result<(), AnalogOutError> {
        if self.change_duty_update.handle_change_duty() {
            self.change_duty_on_cycle()?
        }
        Ok(())
    }
}

impl<'a> AnalogOut<'a> {
    /// Creates a new AnalogOut from a channel, timer, pin number, timer driver, frequency and resolution.
    ///
    /// # Arguments
    /// - `peripheral_channel`: The peripheral channel from the microcontroller
    /// - `timer`: The timer of the PWM signal
    /// - `gpio_pin`: The gpio pin from which the PWM signal will be output
    /// - `timer_driver`: The TimerDriver instance that will handle the interrupts
    /// - `freq_hz`: The frequency of the PWM signal
    /// - `resolution`: An u32 that represents the amount of bits in the desired output resolution. if 0 its set to 1 bit, >= 14
    ///     14 bits of resolution are set  
    ///
    /// # Returns
    /// A result containing the AnalogOut instance or an AnalogOutError
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    pub(crate) fn new(
        peripheral_channel: Peripheral,
        timer: Peripheral,
        gpio_pin: Peripheral,
        timer_driver: TimerDriver<'a>,
        freq_hz: u32,
        resolution: u32,
    ) -> Result<AnalogOut<'a>, AnalogOutError> {
        Ok(AnalogOut {
            inner: SharableRef::new_sharable(_AnalogOut::new(
                peripheral_channel,
                timer,
                gpio_pin,
                timer_driver,
                freq_hz,
                resolution,
            )?),
        })
    }

    /// Creates a new _AnalogOut with a default frecuency of 1000Hz and a resolution of 8bits.
    ///
    /// # Arguments
    /// - `peripheral_channel`: The peripheral channel from the microcontroller
    /// - `timer`: The timer of the PWM signal
    /// - `gpio_pin`: The gpio pin from which the PWM signal will be output
    /// - `timer_driver`: The TimerDriver instance that will handle the interrupts
    ///
    /// # Returns
    /// A result containing the AnalogOut instance or an AnalogOutError
    ///
    /// # Errors
    ///
    /// - `AnalogOutError::InvalidPeripheral`: If any of the peripherals are not from the correct type
    /// - `AnalogOutError::InvalidFrequencyOrDuty`: If the frequency or duty are not compatible
    /// - `AnalogOutError::InvalidArg`: If any of the arguments are not of the correct type
    pub fn default(
        peripheral_channel: Peripheral,
        timer: Peripheral,
        gpio_pin: Peripheral,
        timer_driver: TimerDriver<'a>,
    ) -> Result<AnalogOut<'a>, AnalogOutError> {
        Ok(AnalogOut {
            inner: Rc::new(RefCell::from(_AnalogOut::default(
                peripheral_channel,
                timer,
                gpio_pin,
                timer_driver,
            )?)),
        })
    }
}

impl<'a> InterruptDriver<'a> for AnalogOut<'a> {
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

/// Calculates the duty using the intensity of the signal
///
/// # Arguments
/// - `max_duty` : Maximum duty of the driver instance, that depends of the resolution.
/// - `high_ratio` : Intensity of the signal, from 0 to 1. Is the percentage of time the signal is high.
///  
/// # Returns
/// An u32 value that represents the duty corresponding to the ratio of time the signal is high.  
fn duty_from_high_ratio(max_duty: u32, high_ratio: f32) -> u32 {
    ((max_duty as f32) * high_ratio) as u32
}

impl From<TimerDriverError> for AnalogOutError {
    fn from(value: TimerDriverError) -> Self {
        AnalogOutError::TimerDriverError(value)
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use crate::Microcontroller;

    use super::*;

    fn initialize_test<'a>(pin_num: usize) -> (Microcontroller<'a>, AnalogOut<'a>) {
        let mut micro = Microcontroller::take();
        let out = micro.set_pin_as_default_analog_out(pin_num).unwrap();
        (micro, out)
    }

    #[test]
    fn test0_seting_a_high_ratio_stops_increase_decrease() {
        let (mut micro, mut out) = initialize_test(5);
        out.start_increasing_bounce_back(1, 0.01, 0.0, None)
            .unwrap();
        micro.wait_for_updates(Some(10));
        out.set_high_level_output_ratio(0.0);
        micro.wait_for_updates(Some(10));
        assert_eq!(out.inner.borrow().duty.load(Ordering::Acquire), 0);
        assert_eq!(out.inner.borrow().fixed_change_type, FixedChangeType::None);
    }

    #[test]
    fn test1_increase_bounce_back() {
        let (mut micro, mut out) = initialize_test(5);
        out.start_increasing_bounce_back(1, 0.15, 0.0, Some(1))
            .unwrap();
        micro.wait_for_updates(Some(10));
        assert!(out.inner.borrow().duty.load(Ordering::Acquire) > 0);
        micro.wait_for_updates(Some(10));
        assert!(out.inner.borrow().duty.load(Ordering::Acquire) < 256);
        micro.wait_for_updates(Some(10));
        assert_eq!(out.inner.borrow().duty.load(Ordering::Acquire), 0);
        assert_eq!(out.inner.borrow().fixed_change_type, FixedChangeType::None);
    }
}
