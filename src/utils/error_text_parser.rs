use esp_idf_svc::sys::{EspError, ESP_ERR_INVALID_STATE};

use crate::gpio::DigitalInError;

// use esp_idf_svc::sys::*;

// pub fn err_code_to_text(err_code: i32, context: &str) -> String {
//     match err_code {
//         ESP_ERR_INVALID_ARG => format!("Error: Failed due invalid arguments. Context: {context}"),
//         ESP_ERR_INVALID_STATE => format!("Error: Failed due invalid state. Context: {context}"),
//         _ => format!("Error: This error is not implemented"),
//     }
// }

pub fn map_enable_disable_errors(err: EspError)-> DigitalInError{
    match err.code() {
        ESP_ERR_INVALID_STATE => DigitalInError::StateAlreadySet,
        _ => DigitalInError::InvalidPin,
    }
}