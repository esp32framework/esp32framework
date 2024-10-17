#![allow(clippy::await_holding_refcell_ref)]

mod microcontroller_src;
mod utils;

pub mod ble;
pub mod gpio;
pub mod sensors;
pub mod serial;
pub mod wifi;

pub mod final_project; // TODO: If the folder with the project is moved, delete this line

pub(crate) use microcontroller_src::interrupt_driver::InterruptDriver;

pub use microcontroller_src::Microcontroller;
pub use utils::esp32_framework_error;
pub use utils::timer_driver;
