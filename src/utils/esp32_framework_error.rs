use crate::{
    ble::BleError, gpio::{
        AnalogInError,
        AnalogInPwmError,
        AnalogOutError,
        DigitalInError,
        DigitalOutError,
    }, serial::{I2CError, UARTError},
    utils::timer_driver::TimerDriverError,
    wifi::WifiError
};

/// Represents various error conditions encountered in the ESP32 framework.
#[derive(Debug)]
pub enum Esp32FrameworkError{
    AnalogIn(AnalogInError),
    AnalogInPwm(AnalogInPwmError),
    AnalogOut(AnalogOutError),
    Ble(BleError),
    DigitalIn(DigitalInError),
    DigitalOut(DigitalOutError),
    I2c(I2CError),
    TimerDriver(TimerDriverError),
    Uart(UARTError),
    Wifi(WifiError),
}
