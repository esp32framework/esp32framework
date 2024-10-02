#![allow(clippy::await_holding_refcell_ref)]

mod utils;
mod microcontroller_src;

pub mod ble;
pub mod gpio;
pub mod sensors;
pub mod serial;
pub mod wifi;

pub(crate) use microcontroller_src::interrupt_driver::InterruptDriver;

pub use microcontroller_src::Microcontroller;
pub use utils::timer_driver;
pub use utils::esp32_framework_error;
