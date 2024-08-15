use esp32_nimble::{utilities::BleUuid, BLEAdvertisementData, BLEDevice, NimbleProperties, uuid128};

pub struct BleBeacon {
    uuid: BleUuid,
    name: String,
    ble_device: BLEDevice,
    advertisement: BLEAdvertisementData,
}

pub enum ServiceId{
    StandardService(StandarServiceId),
    ByName(String),
    FromUuid(BleUuid)
}


impl BleBeacon{
    pub fn new(ble_device: BLEDevice, id: ServiceId){
        BleBeacon{,}
    }

    pub fn set_name(name: String){
        
    }

    pub fn add_service(name: String){

    }

    pub fn add_beacon
}

use std::format;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();

    // Configure el servicio y las características que se publicitarán en la publicidad connectionless
    let service_uuid = uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa");

    let mut advertisement = BLEAdvertisementData::new();

    advertisement
        .name("ESP32-Beacon")
        .add_service_uuid(service_uuid)
        .service_data(esp32_nimble::utilities::BleUuid::from_uuid32(0), &[0x5;4]);
    // Configura los datos de publicidad
    ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
        &mut advertisement
    ).unwrap();

    // Empieza la publicidad
    ble_advertising.lock().start().unwrap();

    // Se mantiene el dispositivo publicitando indefinidamente
    loop {
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}