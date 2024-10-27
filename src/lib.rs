#![allow(clippy::await_holding_refcell_ref)]
#![feature(proc_macro_hygiene)]
#![feature(custom_test_frameworks)]
#![feature(test)]
#![test_runner(test_runner_mod::esp_test_runner)]
esp32_testing_macro::use_esp32_tests!(crate::esp_test);

pub mod ble;
pub mod gpio;
mod microcontroller_src;
pub mod sensors;
pub mod serial;
pub mod utils; //TODO private this
pub mod wifi;

pub(crate) use microcontroller_src::interrupt_driver::InterruptDriver;

pub use microcontroller_src::Microcontroller;
pub use utils::esp32_framework_error;
pub use utils::timer_driver;

mod esp_test_runner;


/// The esp_test module, provides a simple way to have a test framework that runs on the microcontroller.
/// After each test the microcontoller will be restarted to guarantee the independence between tests.
pub mod esp_test{
    pub use super::esp_test_runner::*;
    pub use esp32_testing_macro::*;
}
