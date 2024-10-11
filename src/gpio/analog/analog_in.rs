use crate::{
    microcontroller_src::{
        microcontroller::SharableAdcDriver,
        peripherals::{Peripheral, PeripheralError},
    },
    utils::esp32_framework_error::AdcDriverError,
};
use esp_idf_svc::hal::{adc::attenuation::adc_atten_t, adc::*, gpio::*};
use oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver};
use std::{rc::Rc, time::Duration, time::Instant};

const MAX_DIGITAL_VAL: u16 = 4095;

/// Enums the different errors possible when working with the analog in
#[derive(Debug)]
pub enum AnalogInError {
    AdcDriverError(AdcDriverError),
    ChannelCreationError,
    ErrorReading,
    InvalidPeripheral(PeripheralError),
    InvalidPin,
}

/// Driver for receiving analog inputs from a particular pin
/// - `adc_channel_driver`: Instance of AnalogChannels
pub struct AnalogIn<'a> {
    adc_channel_driver: AnalogChannels<'a>,
}

/// Enums the possible channels from the ADC. In the ESP32-C6 the
/// ADC has 7 channels, each on a different GPIO going from
/// GPIO-0 to GPIO-6 inclusive
enum AnalogChannels<'a> {
    Channel0(AdcChannelDriver<'a, Gpio0, Rc<AdcDriver<'a, ADC1>>>),
    Channel1(AdcChannelDriver<'a, Gpio1, Rc<AdcDriver<'a, ADC1>>>),
    Channel2(AdcChannelDriver<'a, Gpio2, Rc<AdcDriver<'a, ADC1>>>),
    Channel3(AdcChannelDriver<'a, Gpio3, Rc<AdcDriver<'a, ADC1>>>),
    Channel4(AdcChannelDriver<'a, Gpio4, Rc<AdcDriver<'a, ADC1>>>),
    Channel5(AdcChannelDriver<'a, Gpio5, Rc<AdcDriver<'a, ADC1>>>),
    Channel6(AdcChannelDriver<'a, Gpio6, Rc<AdcDriver<'a, ADC1>>>),
}

impl<'a> AnalogIn<'a> {
    /// Create a new _AnalogIn for a specific pin.
    ///
    /// # Arguments
    ///
    /// - `pin`: A Peripheral of type Pin
    /// - `adc_driver`: An instance of a SharableAdcDriver
    /// - `attenuation`: An adc_atten_t representing the desired attenuation
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnalogIn` instance, or an `AnalogInError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    pub(crate) fn new(
        pin: Peripheral,
        adc_driver: SharableAdcDriver<'a>,
        attenuation: adc_atten_t,
    ) -> Result<Self, AnalogInError> {
        Ok(AnalogIn {
            adc_channel_driver: AnalogIn::new_channel(pin, adc_driver, attenuation)?,
        })
    }

    /// Creates a new analog channel driver for a given pin
    ///
    /// # Arguments
    ///
    /// - `pin`: A Peripheral of type Pin
    /// - `sharable_adc_driver`: An instance of a SharableAdcDriver
    /// - `attenuation`: An adc_atten_t representing the desired attenuation
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AnalogChannels` instance, or an `AnalogInError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::InvalidPin`: If the pin Peripheral is not valid
    /// - `AnalogInError::ChannelCreationError`: If the channel could not be created
    fn new_channel(
        pin: Peripheral,
        sharable_adc_driver: SharableAdcDriver<'a>,
        attenuation: adc_atten_t,
    ) -> Result<AnalogChannels<'a>, AnalogInError> {
        let mut config = AdcChannelConfig::new();
        config.attenuation = attenuation;
        config.resolution = Resolution::Resolution12Bit;
        config.calibration = true;
        let adc_channel_driver: AnalogChannels<'a> = match pin {
            Peripheral::Pin(pin_num) => match pin_num {
                0 => AnalogChannels::Channel0(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio0::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                1 => AnalogChannels::Channel1(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio1::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                2 => AnalogChannels::Channel2(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio2::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                3 => AnalogChannels::Channel3(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio3::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                4 => AnalogChannels::Channel4(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio4::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                5 => AnalogChannels::Channel5(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio5::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                6 => AnalogChannels::Channel6(
                    AdcChannelDriver::new(sharable_adc_driver, unsafe { Gpio6::new() }, &config)
                        .map_err(|_| AnalogInError::ChannelCreationError)?,
                ),
                _ => return Err(AnalogInError::InvalidPin),
            },
            Peripheral::None => {
                return Err(AnalogInError::InvalidPeripheral(
                    PeripheralError::AlreadyTaken,
                ))
            }
            _ => return Err(AnalogInError::InvalidPeripheral(PeripheralError::NotAPin)),
        };
        Ok(adc_channel_driver)
    }

    /// Returns a digital value read from the analog pin.
    /// The value returned is already attenuated and the range possible depends on the attenuation set.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok having an u16 that represents the read value if the read operation completed successfully,
    /// or an `AnalogInError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::ErrorReading`: If the read operation failed
    pub fn read(&mut self) -> Result<u16, AnalogInError> {
        let mut read_value = match self.adc_channel_driver {
            AnalogChannels::Channel0(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel1(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel2(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel3(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel4(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel5(ref mut channel_driver) => channel_driver.read(),
            AnalogChannels::Channel6(ref mut channel_driver) => channel_driver.read(),
        }
        .map_err(|_| AnalogInError::ErrorReading)?;

        if read_value > MAX_DIGITAL_VAL {
            read_value = MAX_DIGITAL_VAL;
        }
        Ok(read_value)
    }

    /// Returns the raw value read from an analog pin.
    /// The value returned is not attenuated, so its ranges is [0, 4095].
    ///
    /// # Returns
    ///
    /// A `Result` with Ok having an u16 that represents the read raw value if the read operation completed successfully,
    /// or an `AnalogInError` if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::ErrorReading`: If the read operation failed
    pub fn read_raw(&mut self) -> Result<u16, AnalogInError> {
        match self.adc_channel_driver {
            AnalogChannels::Channel0(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel1(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel2(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel3(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel4(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel5(ref mut channel_driver) => channel_driver.read_raw(),
            AnalogChannels::Channel6(ref mut channel_driver) => channel_driver.read_raw(),
        }
        .map_err(|_| AnalogInError::ErrorReading)
    }

    /// Reads multiple times from the analog pin and returns the average value.
    /// It is used to get a more stable value from the analog pin.
    ///
    /// # Arguments
    ///
    /// - `Amount of samples`: The number of times to read from the analog pin and calculate the average value.
    ///
    /// # Returns
    ///
    /// A `result` with Ok containing the u16 that represents the average value if the read operation completed successfully, or an AnalogInError if it fails.
    ///
    /// # Errors
    ///
    /// - `AnalogInError::ErrorReading` : If the read operation fails
    pub fn smooth_read(&mut self, amount_of_samples: u16) -> Result<u16, AnalogInError> {
        let mut smooth_val: u32 = 0;
        for _ in 0..amount_of_samples {
            let read_val = self.read()?;
            smooth_val += read_val as u32;
        }
        let result = smooth_val / amount_of_samples as u32;
        Ok(result as u16)
    }

    pub fn smooth_read_during(&mut self, ms: u16) -> Result<u16, AnalogInError> {
        let mut smooth_val: u64 = 0;
        let duration = Duration::from_millis(ms as u64);
        let starting_time = Instant::now();
        let mut amount_of_samples = 0;
        while starting_time.elapsed() < duration {
            let read_val = self.read()?;
            smooth_val += read_val as u64;
            amount_of_samples += 1;
        }
        let result = smooth_val / amount_of_samples as u64;
        Ok(result as u16)
    }
}

impl From<AdcDriverError> for AnalogInError {
    fn from(value: AdcDriverError) -> Self {
        AnalogInError::AdcDriverError(value)
    }
}
