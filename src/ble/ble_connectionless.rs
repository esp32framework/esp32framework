use esp32_nimble::{utilities::BleUuid, uuid128, BLEAdvertisementData, BLEDevice, BLEError, NimbleProperties};
use uuid::*;
use std::{collections::HashMap, hash::Hash, format, num::NonZero, time::Duration};
use super::StandarServiceId;

const MAX_ADV_PAYLOAD_SIZE: usize = 31;
const PAYLOAD_FIELD_IDENTIFIER_SIZE: usize = 2;

pub struct BleBeacon<'a>{
    advertising_name: String,
    ble_device: &'a mut BLEDevice,
    services: HashMap<ServiceId,Service>,
    advertisement: BLEAdvertisementData,
    time_per_service: Duration
}

#[derive(Clone)]
pub struct Service{
    id: ServiceId,
    data: Vec<u8>
}

#[derive(Debug)]
pub enum BleError{
    ServiceDoesNotFit,
    ServiceTooBig,
    ServiceUnknown,
    StartingFailure,
    Code(u32, String),
}

impl From<BLEError> for BleError{
    fn from(value: BLEError) -> Self {
        match value.code() {
            esp_idf_svc::sys::BLE_HS_EMSGSIZE => BleError::ServiceDoesNotFit,
            _ => BleError::Code(value.code(), value.to_string()),
        }
    }
}

impl Service {
    pub fn new(id: &ServiceId, data: Vec<u8>) -> Result<Service, BleError> {
        let header_bytes = if data.is_empty() {PAYLOAD_FIELD_IDENTIFIER_SIZE} else {PAYLOAD_FIELD_IDENTIFIER_SIZE * 2};
        if data.len() + header_bytes + id.byte_size() > MAX_ADV_PAYLOAD_SIZE {
            Err(BleError::ServiceTooBig)
        } else {
            Ok(Service{id: id.clone(), data})
        }
    }
}

/// in case of repeated name service (using ByName), the first one will be overwritten
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ServiceId {
    StandardService(StandarServiceId),
    ByName(String),
    FromUuid16(u16),
    FromUuid32(u32),
    FromUuid128([u8; 16]),
}


impl Hash for ServiceId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.to_uuid().to_string().hash(state)
    }
}

impl ServiceId {
    pub fn to_uuid(&self) -> BleUuid {
        match self {
            ServiceId::StandardService(service) => {BleUuid::from_uuid16(*service as u16)},
            ServiceId::ByName(name) => {BleUuid::from_uuid128(Uuid::new_v3(&Uuid::NAMESPACE_OID, name.as_bytes()).into_bytes())},
            ServiceId::FromUuid16(uuid) => BleUuid::from_uuid16(*uuid),
            ServiceId::FromUuid32(uuid) => BleUuid::from_uuid32(*uuid),
            ServiceId::FromUuid128(uuid) => BleUuid::from_uuid128(*uuid),
        }
        
    }

    fn byte_size(&self) -> usize{
        match self {
            ServiceId::StandardService(service) => service.byte_size(),
            ServiceId::ByName(_) => {16},
            ServiceId::FromUuid16(_) => 2,
            ServiceId::FromUuid32(_) => 4,
            ServiceId::FromUuid128(_) => 16,
        }
    }
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
    pub fn remove_service(&mut self, service_id: &ServiceId) {
        todo!()
    }
    
    // TODO: change active time with timer
    /// Start advertising one particular service data 
    pub fn advertise_service_data(&mut self, service_id: &ServiceId) -> Result<(), BleError> {
        
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
