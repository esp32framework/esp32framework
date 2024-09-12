use crate::utils::esp32_framework_error::Esp32FrameworkError;


pub trait InterruptDriver {
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError>;
}
