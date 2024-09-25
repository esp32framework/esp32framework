use esp32_nimble::BLEError;

use crate::timer_driver::TimerDriverError;

const ATTRIBUTE_CANNOT_BE_READ: u32 = 258;
const ATTRIBUTE_CANNOT_BE_WRITTEN: u32 = 259;

/// Enums the different errors possible when working with BLE  
#[derive(Debug)]
pub enum BleError {
    AdvertisementError,
    AlreadyConnected,
    CanOnlyBeOneBleDriver,
    CharacteristicNotFound,
    CharacteristicNotNotifiable,
    CharacteristicNotReadable,
    CharacteristicNotWritable,
    Code(u32, String),
    ConnectionError,
    DescriptorNotFound,
    DescriptorNotReadable,
    DescriptorNotWritable,
    DeviceNotFound,
    Disconnected,
    IncorrectHandle,
    InvalidParameters,
    NotFound,
    NotReadable,
    NotWritable,
    PropertiesError,
    ServiceDoesNotFit,
    ServiceNotFound,
    ServiceTooBig,
    ServiceUnknown,
    StartingAdvertisementError,
    StartingFailure,
    StoppingFailure,
    TimeOut,
    TimerDriverError(TimerDriverError),
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
            ATTRIBUTE_CANNOT_BE_READ => BleError::NotReadable,
            ATTRIBUTE_CANNOT_BE_WRITTEN => BleError::NotWritable,
            esp_idf_svc::sys::BLE_HS_CONN_HANDLE_NONE => BleError::NotFound,
            esp_idf_svc::sys::BLE_HS_EDONE => BleError::AlreadyConnected,
            esp_idf_svc::sys::BLE_HS_EMSGSIZE => BleError::ServiceDoesNotFit,
            esp_idf_svc::sys::BLE_HS_ENOTCONN  => BleError::DeviceNotFound,
            _ => BleError::Code(value.code(), value.to_string()),
        }
    }
}

impl BleError {

    /// Creates a more specif BleError from a BLEError, taking into acount its in a service context
    /// 
    /// # Arguments
    /// 
    /// - `value`: The BLEError to transform
    /// 
    /// # Returns
    /// 
    /// The new BleError
    pub fn from_service_context(err: BLEError)-> Self{
        Self::from(err).service_context()
    }
    
    /// Creates a more specif BleError from a BLEError, taking into acount its in a characteristic context
    /// 
    /// # Arguments
    /// 
    /// - `value`: The BLEError to transform
    /// 
    /// # Returns
    /// 
    /// The new BleError
    pub fn from_characteristic_context(err: BLEError) -> Self{
        Self::from(err).characteristic_context()
    }

    /// Creates a more specif BleError from a BLEError, taking into acount its in a descriptors context
    /// 
    /// # Arguments
    /// 
    /// - `value`: The BLEError to transform
    /// 
    /// # Returns
    /// 
    /// The new BleError
    pub fn from_descriptors_context(err: BLEError) -> Self{
        Self::from(err).descriptor_context()
    }

    /// Makes a BleError more specific in the context of services
    fn service_context(self)-> Self{
        match self{
            BleError::DeviceNotFound => BleError::Disconnected,
            BleError::NotFound => BleError::ServiceNotFound,
            _ => self
        }
    }
    
    /// Makes a BleError more specific in the context of characteristic
    fn characteristic_context(self)-> Self{
        match self{
            BleError::DeviceNotFound => BleError::Disconnected,
            BleError::NotFound => BleError::CharacteristicNotFound,
            BleError::NotReadable => BleError::CharacteristicNotReadable,
            BleError::NotWritable => BleError::CharacteristicNotWritable,
            _ => self
        }
    }
    
    /// Makes a BleError more specific in the context of descriptors
    fn descriptor_context(self)-> Self{
        match self{
            BleError::DeviceNotFound => BleError::Disconnected,
            BleError::NotFound => BleError::DescriptorNotFound,
            BleError::NotReadable => BleError::DescriptorNotReadable,
            BleError::NotWritable => BleError::DescriptorNotWritable,
            _ => self
        }
    }
}