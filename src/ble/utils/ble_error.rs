use esp32_nimble::BLEError;

use crate::timer_driver::TimerDriverError;

const ATTRIBUTE_CANNOT_BE_READ: u32 = 258;
const ATTRIBUTE_CANNOT_BE_WRITTEN: u32 = 259;

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
    DescriptorNotFound,
    PropertiesError,
    AdvertisementError,
    StartingAdvertisementError,
    IncorrectHandle,
    ConnectionError,
    InvalidParameters,
    DeviceNotFound,
    AlreadyConnected,
    NotWritable,
    CharacteristicNotWritable,
    DescriptorNotWritable,
    CharacteristicNotNotifiable,
    TimeOut,
    NotReadable,
    DescriptorNotReadable,
    CharacteristicNotReadable,
    CanOnlyBeOneBleDriver,
    Disconnected,
    CouldNotConnectToDevice,
    DeviceNotConnectable,
    InvalidPasskey,
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
            esp_idf_svc::sys::BLE_HS_EINVAL => BleError::InvalidParameters,
            esp_idf_svc::sys::BLE_HS_EDONE => BleError::AlreadyConnected,
            esp_idf_svc::sys::BLE_HS_ENOTCONN  => BleError::DeviceNotFound,
            esp_idf_svc::sys::BLE_HS_ETIMEOUT  => BleError::TimeOut,
            ATTRIBUTE_CANNOT_BE_READ => BleError::NotReadable,
            ATTRIBUTE_CANNOT_BE_WRITTEN => BleError::NotWritable,
            esp_idf_svc::sys::BLE_HS_CONN_HANDLE_NONE => BleError::NotFound,
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

    /// Creates a more specif BleError from a BLEError, taking into acount its in a connection context
    /// 
    /// # Arguments
    /// 
    /// - `value`: The BLEError to transform
    /// 
    /// # Returns
    /// 
    /// The new BleError
    pub fn from_connection_context(err: BLEError)-> Self{
        Self::from(err).connection_params_context()
    }

    /// Creates a more specif BleError from a BLEError, taking into acount its in a connection_params context
    /// 
    /// # Arguments
    /// 
    /// - `value`: The BLEError to transform
    /// 
    /// # Returns
    /// 
    /// The new BleError
    pub fn from_connection_params_context(err: BLEError)-> Self{
        Self::from(err).connection_context()
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
            BleError::NotFound => BleError::ServiceNotFound,
            BleError::DeviceNotFound => BleError::Disconnected,
            _ => self
        }
    }
    
    /// Makes a BleError more specific in the context of characteristic
    fn characteristic_context(self)-> Self{
        match self{
            BleError::NotFound => BleError::CharacteristicNotFound,
            BleError::DeviceNotFound => BleError::Disconnected,
            BleError::NotReadable => BleError::CharacteristicNotReadable,
            BleError::NotWritable => BleError::CharacteristicNotWritable,
            _ => self
        }
    }
    
    /// Makes a BleError more specific in the context of descriptors
    fn descriptor_context(self)-> Self{
        match self{
            BleError::NotFound => BleError::DescriptorNotFound,
            BleError::DeviceNotFound => BleError::Disconnected,
            BleError::NotReadable => BleError::DescriptorNotReadable,
            BleError::NotWritable => BleError::DescriptorNotWritable,
            _ => self
        }
    }

    /// Makes a BleError more specific in the context of descriptors
    fn connection_context(self)-> Self{
        match self{
            BleError::TimeOut => BleError::CouldNotConnectToDevice,
            BleError::DeviceNotFound => BleError::Disconnected,
            _ => self
        }
    }

    /// Makes a BleError more specific in the context of descriptors
    fn connection_params_context(self)-> Self{
        match self{
            BleError::DeviceNotFound => BleError::Disconnected,
            BleError::TimeOut => BleError::ConnectionError,
            _ => self
        }
    }
}