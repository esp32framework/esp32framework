use esp32_nimble::utilities::BleUuid;
use uuid::Uuid;
use std::hash::Hash;

use super::{StandarCharacteristicId, StandarDescriptorId, StandarServiceId};

/// Enums the possible types of Ids:
/// - `StandardService`: The UUIDs of standard Bluetooth Low Energy (BLE) services.
/// - `StandarCharacteristic`: The UUIDs of standard Bluetooth Low Energy (BLE) characteristics.
/// - `ByName`: A string that can be made into a BLE id.
/// - `FromUuid16`: A way to get a BLE id from an u16.
/// - `FromUuid32`: A way to get a BLE id from an u32.
/// - `FromUuid128`: A way to get a BLE id from an [u8;16].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BleId {
    StandardService(StandarServiceId),
    StandarCharacteristic(StandarCharacteristicId),
    StandarDescriptor(StandarDescriptorId),
    ByName(String),
    FromUuid16(u16),
    FromUuid32(u32),
    FromUuid128([u8; 16]),
}


impl Hash for BleId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_uuid().to_string().hash(state)
    }
}

impl From<BleUuid> for BleId{
    fn from(value: BleUuid) -> Self {
        match value{
            BleUuid::Uuid16(id) => BleId::FromUuid16(id),
            BleUuid::Uuid32(id) => BleId::FromUuid32(id),
            BleUuid::Uuid128(id) => BleId::FromUuid128(id),
        }
    }
}

impl From<&BleUuid> for BleId{
    fn from(value: &BleUuid) -> Self {
        Self::from(*value)
    }
}


impl BleId {
    /// Creates a BleUuid from a BleId
    /// 
    /// # Returns
    /// 
    /// The corresponfing BleUuid
    pub fn to_uuid(&self) -> BleUuid {
        match self {
            BleId::StandardService(service) => {BleUuid::from_uuid16(*service as u16)},
            BleId::StandarCharacteristic(characteristic) => {BleUuid::from_uuid16(*characteristic as u16)},
            BleId::StandarDescriptor(descriptor) => {BleUuid::from_uuid16(*descriptor as u16)},
            BleId::FromUuid16(uuid) => BleUuid::from_uuid16(*uuid),
            BleId::FromUuid32(uuid) => BleUuid::from_uuid32(*uuid),
            BleId::FromUuid128(uuid) => BleUuid::from_uuid128(*uuid),
            BleId::ByName(name) => {
                let arr: [u8;4] = Uuid::new_v3(&Uuid::NAMESPACE_OID, name.as_bytes()).into_bytes()[0..4].try_into().unwrap();
                BleUuid::from_uuid32(u32::from_be_bytes(arr))
            },
        }
    }

    /// Gets the byte size
    /// 
    /// # Returns
    /// 
    /// The usize representing the byte size
    pub fn byte_size(&self) -> usize {
        match self {
            BleId::StandardService(service) => service.byte_size(),
            BleId::StandarCharacteristic(characteristic) => characteristic.byte_size(),
            BleId::StandarDescriptor(descriptor) => descriptor.byte_size(),
            BleId::ByName(_) => 4,
            BleId::FromUuid16(_) => 2,
            BleId::FromUuid32(_) => 4,
            BleId::FromUuid128(_) => 16,
        }
    }
}