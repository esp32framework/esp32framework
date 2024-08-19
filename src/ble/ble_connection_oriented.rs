use std::collections::HashMap;

use esp32_nimble::{utilities::mutex::Mutex, uuid128, BLEAdvertisementData, BLEAdvertising, BLEDevice, BLEServer, NimbleProperties};
use esp_idf_svc::hal::task;

use super::{BleError, BleId, Characteristic, Service};

pub struct BleServer<'a> {
    advertising_name: String,
    ble_server: &'a mut BLEServer,
    services: Vec<Service>, 
    advertisement: &'a Mutex<BLEAdvertising>,
}

impl <'a>BleServer<'a> {

    pub fn new(name: String, ble_device: &mut BLEDevice, services: Vec<Service> ) -> Self {
        let mut server = BleServer{
            advertising_name: name,
            ble_server: ble_device.get_server(),
            services: services.clone(),
            advertisement: ble_device.get_advertising(),
        };
            
        for service in  &services {
            server.add_service(service);
        }

        server
    }

    pub fn connection_handler() {

    }

    pub fn disconnection_handler() {

    }

    /// Add or overwrite a service to the server
    pub fn add_service(&mut self, service: &Service){
        self.ble_server.create_service(service.id.to_uuid());

        for characteristic in &service.characteristics{
            self.add_characteristic(service.id.clone(), characteristic);
        }
    }

    /// add a characteristic to the server. Returns an error if the service does not exist or the properties are invalid
    pub fn add_characteristic(&mut self, service_id: BleId, characteristic: &Characteristic) -> Result<(), BleError> {
        let server_service = task::block_on(async {
            self.ble_server.get_service(service_id.to_uuid()).await
        });
        if let Some(service) = server_service {    
            match NimbleProperties::from_bits(characteristic.properties.to_be()) {
                Some(properties) => {
                    service.lock().create_characteristic(
                        characteristic.id.to_uuid(),
                        properties,
                    );
                    return Ok(());
                },
                None => {return Err(BleError::PropertiesError)},
            }
        }
        Err(BleError::ServiceNotFound)
    }

    pub fn start(&mut self) -> Result<(), BleError>{
        self.create_advertisement_data()?;
        self.advertisement.lock().start().map_err(|_| BleError::StartingAdvertisementError)
    }

    fn create_advertisement_data(&mut self) -> Result<(), BleError>{
        let mut adv_data = BLEAdvertisementData::new();
        adv_data.name(&self.advertising_name);
        for service in &self.services {
            adv_data.add_service_uuid(service.id.to_uuid());
        }
        self.advertisement.lock().set_data(&mut adv_data).map_err(|_| BleError::AdvertisementError)
    }
}

