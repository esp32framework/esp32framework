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

#[derive(Debug)]
pub enum Esp32FrameworkError{
    AnalogInError(AnalogInError),
    AnalogInPwmError(AnalogInPwmError),
    AnalogOutError(AnalogOutError),
    DigitalInError(DigitalInError),
    DigitalOutError(DigitalOutError),
    TimerDriverError(TimerDriverError)
}