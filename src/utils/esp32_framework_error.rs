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
    AnalogIn(AnalogInError),
    AnalogInPwm(AnalogInPwmError),
    AnalogOut(AnalogOutError),
    DigitalIn(DigitalInError),
    DigitalOut(DigitalOutError),
    TimerDriver(TimerDriverError)
}