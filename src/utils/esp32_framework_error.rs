use crate::{
    ble::BleError, gpio::{
        AnalogInError,
        AnalogInPwmError,
        AnalogOutError,
        DigitalInError,
        DigitalOutError,
    }, serial::{I2CError, UARTError},
    utils::timer_driver::TimerDriverError,
    wifi::wifi::WifiError
};

#[derive(Debug)]
pub enum Esp32FrameworkError{
    AnalogIn(AnalogInError),
    AnalogInPwm(AnalogInPwmError),
    AnalogOut(AnalogOutError),
    DigitalIn(DigitalInError),
    DigitalOut(DigitalOutError),
    TimerDriver(TimerDriverError),
    I2c(I2CError),
    Uart(UARTError),
    Ble(BleError),
    Wifi(WifiError),
}