use esp32_nimble::{utilities::BleUuid, uuid128, BLEAdvertisementData, BLEDevice, BLEError, NimbleProperties};
use uuid::*;
use std::{collections::HashMap, hash::Hash, format, num::NonZero, time::Duration};
use super::{StandarServiceId, Service, BleId, BleError};

pub struct BleBeacon<'a>{
    advertising_name: String,
    ble_device: &'a mut BLEDevice,
    services: HashMap<BleId,Service>,
    advertisement: BLEAdvertisementData,
    time_per_service: Duration
}


impl <'a>BleBeacon<'a>{
    pub fn new(ble_device: &'a mut BLEDevice, advertising_name: String, services: Vec<Service>) -> Result<Self, BleError>{
        let mut advertisement = BLEAdvertisementData::new();
        advertisement.name(&advertising_name);
        let mut beacon = BleBeacon{advertising_name, 
            ble_device, 
            services: HashMap::new(), 
            advertisement, 
            time_per_service: Duration::from_secs(0)};
        beacon.add_services(services)?;
        Ok(beacon)
    }

    pub fn set_name(&mut self, name: String) -> &mut Self{
        self.advertising_name = name;
        self
    }

    fn set_advertising_data(&mut self)->Result<(), BleError>{
        let mut ble_adv = self.ble_device.get_advertising().lock();
        loop{
            let res: Result<(), BLEError> = ble_adv.advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(&mut self.advertisement);
            if  BLEError::convert(esp_idf_svc::sys::BLE_HS_EBUSY) != res {
                return res.map_err(BleError::from);
            }
        }
    }

    pub fn add_service(&mut self, service: Service) -> Result<&mut Self, BleError>{
        self.advertisement.add_service_uuid(service.id.to_uuid());
        if !service.data.is_empty(){
            self.advertisement.service_data(service.id.to_uuid(), &service.data);
        } 
        self.set_advertising_data()?;
        self.services.insert(service.id.clone(), service);
        Ok(self)
    }

    pub fn add_services(&mut self, services: Vec<Service>) -> Result<(), BleError>{
        for service in services{
            self.add_service(service)?;
        }
        Ok(())
    }

    // check if advertisement allows removing service
    pub fn remove_service(&mut self, service_id: &BleId) {
        todo!()
    }
    
    // TODO: change active time with timer
    /// Start advertising one particular service data 
    pub fn advertise_service_data(&mut self, service_id: &BleId) -> Result<(), BleError> {
        
        match self.services.get(service_id){
            Some(request_service) => {
                self.advertisement.service_data(request_service.id.to_uuid(), &request_service.data);
                self.set_advertising_data()?;
                self.start()
            },
            None => Err(BleError::ServiceUnknown),
        }
    }

    pub fn start(&self) -> Result<(), BleError>{
        let mut ble_adv = self.ble_device.get_advertising().lock();
        ble_adv.start().map_err(|_| BleError::StartingFailure)
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
//         .service_data(BleUuid::from_uuid32(0), &[0x5;4]);
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
