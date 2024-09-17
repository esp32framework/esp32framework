use esp32_nimble::BLEError;

use crate::timer_driver::TimerDriverError;

const ATTRIBUTE_CANNOT_BE_READ: u32 = 258;

/// Enums the different errors possible when working with BLE  
#[derive(Debug)]
pub enum BleError{
    ServiceDoesNotFit,
    ServiceTooBig,
    ServiceUnknown,
    StartingFailure,
    StoppingFailure,
    TimerDriverError(TimerDriverError),
    Code(u32, String),
    NotFound,
    ServiceNotFound,
    CharacteristicNotFound,
    PropertiesError,
    AdvertisementError,
    StartingAdvertisementError,
    IncorrectHandle,
    ConnectionError,
    InvalidParameters,
    DeviceNotFound,
    AlreadyConnected,
    CharacteristicIsNotReadable,
    CharacteristicIsNotWritable,
    CharacteristicIsNotNotifiable,
    TimeOut,
    NotReadable,
    Disconnected
}

impl From<BLEError> for BleError {

    /// Creates a BleError from a BLEError
    /// 
    /// # Arguments
    /// 
    /// - `value`: The BLEError to transform
    /// 
    /// # Returns
    /// 
    /// The new BleError
    fn from(value: BLEError) -> Self {
        match value.code() {
            esp_idf_svc::sys::BLE_HS_EMSGSIZE => BleError::ServiceDoesNotFit,
            esp_idf_svc::sys::BLE_HS_EDONE => BleError::AlreadyConnected,
            esp_idf_svc::sys::BLE_HS_ENOTCONN  => BleError::DeviceNotFound,
            ATTRIBUTE_CANNOT_BE_READ => BleError::NotReadable,
            esp_idf_svc::sys::BLE_HS_CONN_HANDLE_NONE => BleError::NotFound,
            _ => BleError::Code(value.code(), value.to_string()),
        }
    }
}

impl BleError {

    pub fn from_service_context(err: BLEError)-> Self{
        Self::from(err).service_context()
    }
    
    pub fn from_characteristic_context(err: BLEError) -> Self{
        Self::from(err).characteristic_context()
    }

    fn service_context(self)-> Self{
        match self{
            BleError::NotFound => BleError::ServiceNotFound,
            _ => self
        }
    }

    fn characteristic_context(self)-> Self{
        match self{
            BleError::NotFound => BleError::CharacteristicNotFound,
            _ => self
        }
    }
}