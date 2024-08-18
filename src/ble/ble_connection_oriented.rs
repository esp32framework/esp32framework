use std::collections::HashMap;

use esp32_nimble::{BLEAdvertisementData, BLEDevice};

use super::{Service, BleId};

pub struct BleServer<'a> {
    advertising_name: String,
    ble_device: &'a mut BLEDevice,
    services: HashMap<BleId, Service>,
    advertisement: BLEAdvertisementData,

}

impl <'a>BleServer<'a> {
    pub fn new() -> Self {

    }

    pub fn connection_handler() {

    }

    pub fn disconnection_handler() {

    }

    pub fn add_service(){

    }

    pub fn add_characteristic(){
        
    }
}