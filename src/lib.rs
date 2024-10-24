#![allow(clippy::await_holding_refcell_ref)]
#![feature(proc_macro_hygiene)]
#![feature(custom_test_frameworks)]
#![test_runner(test::esp32_test_runner)]
#![feature(test)]

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

#[cfg(test)]
mod test;

