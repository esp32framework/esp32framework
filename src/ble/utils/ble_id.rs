use esp32_nimble::utilities::BleUuid;
use std::hash::Hash;
use uuid::Uuid;

use super::ble_standard_uuids::{StandardCharacteristicId, StandardDescriptorId, StandardServiceId};

/// Enums the possible types of Ids:
/// - `FromUuid16`: A way to get a BLE id from an `u16`.
/// - `FromUuid128`: A way to get a BLE id from an `[u8;16]`.
#[derive(Debug, Clone)]
pub enum BleId {
    FromUuid16(u16),
    FromUuid128([u8; 16]),
}

impl PartialEq for BleId {
    fn eq(&self, other: &Self) -> bool {
        self.to_uuid() == other.to_uuid()
    }
}

impl Eq for BleId {}

impl Hash for BleId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_uuid().to_string().hash(state)
    }
}

impl From<BleUuid> for BleId {
    fn from(value: BleUuid) -> Self {
        match value {
            BleUuid::Uuid16(id) => BleId::FromUuid16(id),
            BleUuid::Uuid32(id) => {
                let mut output: [u8; 16] = [0; 16];
                let input = id.to_le_bytes();
                output[..4].copy_from_slice(&input);
                BleId::FromUuid128(output)
            }
            BleUuid::Uuid128(id) => BleId::FromUuid128(id),
        }
    }
}

impl From<&BleUuid> for BleId {
    fn from(value: &BleUuid) -> Self {
        Self::from(*value)
    }
}

impl BleId {
    /// Creates a `BleId::FromUuid16` from a StandardService
    ///
    /// # Arguments
    ///
    /// - `StandardServiceId`: A standard service Id to be convert into a BleId
    ///
    /// # Returns
    ///
    /// A new BleId
    pub const fn from_standard_service(id: StandardServiceId) -> BleId {
        BleId::FromUuid16(id as u16)
    }

    /// Creates a `BleId::FromUuid16` from a StandardService
    ///
    /// # Arguments
    ///
    /// - `StandardCharacteristicId`: A standard characteristic Id to be convert into a BleId
    ///
    /// # Returns
    ///
    /// A new BleId
    pub const fn from_standard_characteristic(id: StandardCharacteristicId) -> BleId {
        BleId::FromUuid16(id as u16)
    }

    /// Creates a `BleId::FromUuid16` from a StandardService
    ///
    /// # Arguments
    ///
    /// - `StandardDescriptorId`: A standard descriptor Id to be convert into a BleId
    ///
    /// # Returns
    ///
    /// A new BleId
    pub const fn from_standard_descriptor(id: StandardDescriptorId) -> BleId {
        BleId::FromUuid16(id as u16)
    }

    /// Creates a `BleId::FromUuid16` from a `&str`
    ///
    /// # Arguments
    ///
    /// - `name`: A string to be mapped into a `BleId::FromUuid16`
    ///
    /// # Returns
    ///
    /// A new BleId
    pub fn from_name(name: &str)-> BleId{
        let arr: [u8; 2] = Uuid::new_v3(&Uuid::NAMESPACE_OID, name.as_bytes()).into_bytes()
                    [0..2]
                    .try_into()
                    .unwrap();
        BleId::FromUuid16(u16::from_be_bytes(arr))
    }

    /// Creates a BleUuid from a BleId
    ///
    /// # Returns
    ///
    /// The corresponfing BleUuid
    pub(crate) fn to_uuid(&self) -> BleUuid {
        match self {
            BleId::FromUuid16(uuid) => BleUuid::from_uuid16(*uuid),
            BleId::FromUuid128(uuid) => BleUuid::from_uuid128(*uuid)
        }
    }

    /// Gets the byte size
    ///
    /// # Returns
    ///
    /// The usize representing the byte size
    pub fn byte_size(&self) -> usize {
        match self {
            BleId::FromUuid16(_) => 2,
            BleId::FromUuid128(_) => 16,
        }
    }
}
