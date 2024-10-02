use crate::{
    gpio::digital_in::{DigitalIn, DigitalInError},
    microcontroller_src::peripherals::Peripheral,
    timer_driver::TimerDriverError,
    utils::timer_driver::TimerDriver,
};
use esp_idf_svc::hal::ledc::config::TimerConfig;

const FREQUENCY_TO_SAMPLING_RATIO: u32 = 2;

/// Enums the different errors possible when working with the analog in with pwm signals
#[derive(Debug)]
pub enum AnalogInPwmError {
    DigitalDriverError(DigitalInError),
    TimerDriverError(TimerDriverError),
}

/// Driver for receiving analog input with a PWM signal from a particular DigitalIn.
/// - `digital_in`: A DigitalIn used to receive the digital signals
/// - `sampling`: An u32 representing the quantity of signal samples for each read
pub struct AnalogInPwm<'a> {
    digital_in: DigitalIn<'a>,
    sampling: u32,
}

impl<'a> AnalogInPwm<'a> {
    /// Create a new AnalogInPwm for a specific pin.
    /// The Frecuency to Sampling ratio is defined in 2 by default
    ///
    /// # Arguments
    ///
    /// - `timer_driver`: A TimerDriver instance
    /// - `per`: A Peripheral capable of being transformed into an AnyIOPin
    /// - `frequency_hz`: An u32 representing the frequency in hertz of signal to be read.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnalogInPwm` instance, or an `AnalogInPwmError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInPwmError::DigitalDriverError`: If the creation of the DigitalIn fails
    pub fn new(
        timer_driver: TimerDriver<'a>,
        per: Peripheral,
        frequency_hz: u32,
    ) -> Result<Self, AnalogInPwmError> {
        let digital_in = DigitalIn::new(timer_driver, per, None)
            .map_err(AnalogInPwmError::DigitalDriverError)?;
        Ok(AnalogInPwm {
            digital_in,
            sampling: FREQUENCY_TO_SAMPLING_RATIO * frequency_hz,
        })
    }

    /// Create a new AnalogInPwm with a default frecuency of 1000Hz.
    ///
    /// # Arguments
    ///
    /// - `timer_driver`: A TimerDriver instance
    /// - `per`: A Peripheral capable of being transformed into an AnyIOPin
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnalogInPwm` instance, or an `AnalogInPwmError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInPwmError::DigitalDriverError`: If the creation of the DigitalIn fails
    pub fn default(
        timer_driver: TimerDriver<'a>,
        per: Peripheral,
    ) -> Result<Self, AnalogInPwmError> {
        Self::new(timer_driver, per, TimerConfig::new().frequency.into())
    }

    /// Changes the frequency between each read.
    ///
    /// # Arguments
    ///
    /// - `frequency_hz`: An u32 representing the frequency in hertz of signal to be read.
    pub fn set_sampling_frequency(&mut self, frequency_hz: u32) {
        self.sampling = FREQUENCY_TO_SAMPLING_RATIO * frequency_hz
    }

    /// Returns the intensity value [0 , 1] obtained dividing the amount
    /// of Highs read by the amount of samples taken.
    ///
    /// # Returns
    ///
    /// An f32 representing the intensity read
    pub fn read(&self) -> f32 {
        let mut highs: u32 = 0;
        for _num in 0..(self.sampling) {
            if self.digital_in.is_high() {
                highs += 1
            }
        }
        (highs as f32) / (self.sampling as f32)
    }

    /// Returns the intensity value using percentage.
    ///
    /// # Returns
    ///
    /// An f32 representing the percentage value
    pub fn read_percentage(&self) -> f32 {
        self.read() * 100.0
    }
}

impl From<TimerDriverError> for AnalogInPwmError {
    fn from(value: TimerDriverError) -> Self {
        AnalogInPwmError::TimerDriverError(value)
    }
}
