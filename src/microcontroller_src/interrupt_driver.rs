use crate::utils::esp32_framework_error::Esp32FrameworkError;

/// This trait is for drivers that will have on top of the referece the user holds, a reference stored in the
/// microcontroller. This is so upong interrupts, the microcontroller can execute the corresponding updates by
/// calling `update_interrupt`
pub(crate) trait InterruptDriver<'a> {
    /// This function will be called in order to update a given driver
    ///
    /// #Returns
    ///
    /// If the update was successfull `Ok(())` is returned. If not `Err(Esp32FrameworkError)` with
    /// the corresponding driver error type is returned.
    fn update_interrupt(&mut self) -> Result<(), Esp32FrameworkError>;

    /// This function returns an updater of the Interrupt driver. This may be a reference to the original
    /// driver or a completly diferent struct that implements the `InterruptDriver` trait
    fn get_updater(&self) -> Box<dyn InterruptDriver<'a> + 'a>;
}
