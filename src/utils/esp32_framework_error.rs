use esp_idf_svc::sys::EspError;

use crate::{
    ble::BleError,
    gpio::{
        analog::{AnalogInError, AnalogInPwmError, AnalogOutError},
        digital::{DigitalInError, DigitalOutError},
    },
    microcontroller_src::peripherals::PeripheralError,
    serial::{i2c::I2CError, uart::UARTError},
    utils::timer_driver::TimerDriverError,
    wifi::WifiError,
};

/// Represents various error conditions encountered in the ESP32 framework.
#[derive(Debug)]
pub enum Esp32FrameworkError {
    AnalogIn(AnalogInError),
    AnalogInPwm(AnalogInPwmError),
    AnalogOut(AnalogOutError),
    Ble(BleError),
    CantHaveMoreThanOneMicrocontroller,
    DigitalIn(DigitalInError),
    DigitalOut(DigitalOutError),
    I2c(I2CError),
    TimerDriver(TimerDriverError),
    Uart(UARTError),
    Wifi(WifiError),
}

/// A macro to implement the `From` trait for converting different error types into
/// the `Esp32FrameworkError` enum.
///
/// # Parameters
///
/// This macro accepts a list of pairs of variant names (which represent the variants
/// of the `Esp32FrameworkError` enum) and error types.
macro_rules! impl_from_for_esp32_error {
    ($( $variant:ident => $error_type:ty ),* $(,)?) => {
        $(
            impl From<$error_type> for Esp32FrameworkError {
                fn from(value: $error_type) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

impl_from_for_esp32_error! {
    AnalogIn => AnalogInError,
    AnalogInPwm => AnalogInPwmError,
    AnalogOut => AnalogOutError,
    Ble => BleError,
    DigitalIn => DigitalInError,
    DigitalOut => DigitalOutError,
    I2c => I2CError,
    TimerDriver => TimerDriverError,
    Uart => UARTError,
    Wifi => WifiError,
}

#[derive(Debug)]
pub enum AdcDriverError {
    AlreadyTaken,
    Code(i32, String),
    ClockError,
    InvalidArgs,
    NoMemory,
    PeripheralError(PeripheralError),
}

impl From<EspError> for AdcDriverError {
    fn from(value: EspError) -> Self {
        match value.code() {
            esp_idf_svc::sys::ESP_ERR_INVALID_ARG => AdcDriverError::InvalidArgs,
            esp_idf_svc::sys::ESP_ERR_NO_MEM => AdcDriverError::NoMemory,
            esp_idf_svc::sys::ESP_ERR_NOT_FOUND => AdcDriverError::AlreadyTaken,
            esp_idf_svc::sys::ESP_FAIL => AdcDriverError::ClockError,
            _ => AdcDriverError::Code(value.code(), value.to_string()),
        }
    }
}

impl From<PeripheralError> for AdcDriverError {
    fn from(value: PeripheralError) -> Self {
        AdcDriverError::PeripheralError(value)
    }
}
