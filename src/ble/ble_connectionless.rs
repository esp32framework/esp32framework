use esp32_nimble::{utilities::BleUuid, uuid128, BLEAdvertisementData, BLEDevice, BLEError, NimbleProperties};
use std::{format, num::NonZero, time::Duration};

use super::StandarServiceId;

pub struct BleBeacon<'a>{
    advertising_name: String,
    ble_device: &'a mut BLEDevice,
    services: Vec<Service>,
    advertisement: BLEAdvertisementData,
    time_per_service: Duration
}

pub struct Service{
    id: ServiceId,
    uuid: BleUuid,
    data: Vec<u8>
}

impl Service{
    //TODO: check size of data
    pub fn new(id: &ServiceId, data: Vec<u8>) -> Service {
        let uuid: BleUuid = match id {
            ServiceId::StandardService(service) => {esp32_nimble::utilities::BleUuid::from_uuid16(*service as u16)},
            ServiceId::ByName(name) => {todo!()},
            ServiceId::FromUuid16(uuid) => esp32_nimble::utilities::BleUuid::from_uuid16(*uuid),
            ServiceId::FromUuid32(uuid) => esp32_nimble::utilities::BleUuid::from_uuid32(*uuid),
            ServiceId::FromUuid128(uuid) => esp32_nimble::utilities::BleUuid::from_uuid128(*uuid),
        };
        Service{id: id.clone(), uuid, data}
    }
}

/// in case of repeated name service (using ByName), the first one will be overwritten
#[derive(PartialEq, Clone)]
pub enum ServiceId{
    StandardService(StandarServiceId),
    ByName(String),
    FromUuid16(u16),
    FromUuid32(u32),
    FromUuid128([u8; 16]),
}


impl <'a>BleBeacon<'a>{
    pub fn new(ble_device: &'a mut BLEDevice, advertising_name: String, services: Vec<Service>) -> BleBeacon<'a>{
        let mut advertisement = BLEAdvertisementData::new();
        advertisement.name(&advertising_name);
        BleBeacon{advertising_name, ble_device, services, advertisement, time_per_service: Duration::from_secs(0)}
    }

    pub fn set_name(&mut self, name: String) -> &mut Self{
        self.advertising_name = name;
        self
    }

    pub fn add_service(&mut self, service: Service) ->  &mut Self{
        self.services.push(service);
        self
    }

    // TODO: vec<Service> || HashMap(ServiceId, Service)
    // TODO: change active time with timer
    pub fn start_advertisement(&mut self, service_id: &ServiceId, service_active_time: Duration) -> Result<(), BLEError> {
        let ble_advertising = self.ble_device.get_advertising();

        if let Some(request_service) = self.services.iter().find(|service| service.id == *service_id){
            self.advertisement.service_data(request_service.uuid, &request_service.data);
            ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
                &mut self.advertisement
            )?;
            return ble_advertising.lock().start()     
        }
        return BLEError::convert(esp_idf_svc::sys::BLE_HS_EINVAL);
    }

}


// fn main() {
//     esp_idf_svc::sys::link_patches();
//     esp_idf_svc::log::EspLogger::initialize_default();

//     let ble_device = BLEDevice::take();
//     let ble_advertising = ble_device.get_advertising();

//     // Configure el servicio y las características que se publicitarán en la publicidad connectionless
//     let service_uuid = uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa");

//     let mut advertisement = BLEAdvertisementData::new();

//     advertisement
//         .name("ESP32-Beacon")
//         .add_service_uuid(service_uuid)
//         .service_data(esp32_nimble::utilities::BleUuid::from_uuid32(0), &[0x5;4]);
//     // Configura los datos de publicidad
//     ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
//         &mut advertisement
//     ).unwrap();

//     // Empieza la publicidad
//     ble_advertising.lock().start().unwrap();

//     // Se mantiene el dispositivo publicitando indefinidamente
//     loop {
//         esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
//     }
// }
