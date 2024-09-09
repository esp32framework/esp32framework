use crate::{
    utils::timer_driver::TimerDriverError,
    gpio::{
        AnalogInError,
        AnalogInPwmError,
        AnalogOutError,
        DigitalInError,
        DigitalOutError,

    }
};

/// Represents various error conditions encountered in the ESP32 framework.
#[derive(Debug)]
pub enum Esp32FrameworkError{
    AnalogIn(AnalogInError),
    AnalogInPwm(AnalogInPwmError),
    AnalogOut(AnalogOutError),
    DigitalIn(DigitalInError),
    DigitalOut(DigitalOutError),
    TimerDriver(TimerDriverError)
}
