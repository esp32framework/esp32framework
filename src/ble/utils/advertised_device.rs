use esp32_nimble::{
    enums::{AdvFlag, AdvType},
    BLEAddress, BLEAdvertisedDevice,
};

use super::BleId;

pub struct BleAdvertisedDevice {
    device: BLEAdvertisedDevice,
}

impl BleAdvertisedDevice {
    /// Gets the name of the device
    ///
    /// # Returns
    ///
    /// A String representing the name
    pub fn name(&self) -> String {
        self.device.name().to_string()
    }

    /// Get the address of the advertising device.
    ///
    /// # Returns
    ///
    /// A BLEAddress
    pub fn addr(&self) -> &BLEAddress {
        self.device.addr()
    }

    /// Get the advertisement type.
    ///
    /// # Returns
    ///
    /// A AdvType
    pub fn adv_type(&self) -> AdvType {
        self.device.adv_type()
    }

    /// Get the advertisement flags.
    ///
    /// # Returns
    ///
    /// An `Option` with the AdvFlag if there is one, otherwise `None`
    pub fn adv_flags(&self) -> Option<AdvFlag> {
        self.device.adv_flags()
    }

    /// Get the rssi
    ///
    /// # Returns
    ///
    /// The rssi
    pub fn rssi(&self) -> i32 {
        self.device.rssi()
    }

    /// Get all the service uuids
    ///
    /// # Returns
    ///
    /// A vector of BleId containing every service id
    pub fn get_service_uuids(&self) -> Vec<BleId> {
        self.device.get_service_uuids().map(BleId::from).collect()
    }

    /// Indicates whether a service is being advertised or not
    ///
    /// # Arguments
    ///
    /// - `id`: The BleId of the service in doubt
    ///
    /// # Returns
    ///
    /// True if the service is being advertised, False if not
    pub fn is_advertising_service(&self, id: &BleId) -> bool {
        self.get_service_uuids().contains(id)
    }

    /// Gets the data of every service contained
    ///
    /// # Returns
    ///
    /// A vector of tuple. Each tuple has the BleId of a service and its data which is a slice of bytes
    pub fn get_service_data_list(&self) -> Vec<(BleId, &[u8])> {
        self.device
            .get_service_data_list()
            .map(|s| (BleId::from(s.uuid()), s.data()))
            .collect()
    }

    /// Get the data of a specific service
    ///
    /// # Arguments
    ///
    /// - `id`: The BleId of the service to be searched
    ///
    /// # Returns
    ///
    /// An `Option` with a tuple containing the BleId of the service and its data in a slice of bytes, `None` if
    /// the service is not on the device
    pub fn get_service_data(&self, id: BleId) -> Option<(BleId, &[u8])> {
        self.get_service_data_list().into_iter().find(|s| s.0 == id)
    }

    /// Gets the manufacture data of th device
    ///
    /// # Returns
    ///
    /// An `Option` with the data if there is one, `None` if there is not
    pub fn get_manufacture_data(&self) -> Option<&[u8]> {
        self.device.get_manufacture_data()
    }

    /// Returns wether or not a device is connectable acording to the advertisement type
    pub fn is_connectable(&self) -> bool {
        adv_type_is_connectable(&self.adv_type())
    }
}

impl From<&BLEAdvertisedDevice> for BleAdvertisedDevice {
    fn from(value: &BLEAdvertisedDevice) -> Self {
        BleAdvertisedDevice {
            device: value.clone(),
        }
    }
}

/// Returns wether or not a device is connectable acording to the advertisement type
fn adv_type_is_connectable(adv_type: &AdvType) -> bool {
    match adv_type {
        AdvType::Ind => true,
        AdvType::DirectInd => true,
        AdvType::ScanInd => false,
        AdvType::NonconnInd => false,
    }
}
