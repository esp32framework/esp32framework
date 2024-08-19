use esp32_nimble::{utilities::{mutex::Mutex, BleUuid}, uuid128, BLEAdvertisementData, BLEAdvertising, BLEDevice, BLEError, NimbleProperties};
use esp_idf_svc::hal::timer::Timer;
use uuid::*;
use std::{cell::{RefCell, RefMut}, collections::HashMap, format, hash::Hash, num::NonZero, rc::Rc, time::Duration, u64};
use crate::utils::timer_driver::{TimerDriver, TimerDriverError};

use super::StandarServiceId;

const MAX_ADV_PAYLOAD_SIZE: usize = 31;
const PAYLOAD_FIELD_IDENTIFIER_SIZE: usize = 2;

pub struct BleBeacon<'a>{
    advertising_name: String,
    ble_device: &'a mut BLEDevice,
    services: HashMap<ServiceId,Service>,
    advertisement: Rc<RefCell<BLEAdvertisementData>>,
    timer_driver: TimerDriver<'a>,
    time_per_service: Duration,
    looping_service_data: bool
}

#[derive(Clone, Debug)]
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
    StoppingFailure,
    TimerDriverError(TimerDriverError),
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
    pub fn new(ble_device: &'a mut BLEDevice, timer_driver: TimerDriver<'a>, advertising_name: String, services: Vec<Service>) -> Result<Self, BleError>{
        let mut advertisement = BLEAdvertisementData::new();
        advertisement.name(&advertising_name);
        let mut beacon = BleBeacon{
            advertising_name, 
            ble_device, 
            services: HashMap::new(), 
            advertisement: Rc::new(RefCell::from(advertisement)),
            timer_driver,
            time_per_service: Duration::from_secs(1),
            looping_service_data: false
        };
        beacon.add_services(services)?;
        Ok(beacon)
    }

    fn advertisement(&mut self)-> RefMut<BLEAdvertisementData>{
        self.advertisement.borrow_mut()
    }

    pub fn set_name(&mut self, name: String) -> &mut Self{
        self.advertisement().name(name.as_str());
        self.advertising_name = name;
        self
    }
    
    fn insert_service(&mut self, service: Service){
        self.advertisement().add_service_uuid(service.id.to_uuid());
        if !service.data.is_empty(){
            self.advertisement().service_data(service.id.to_uuid(), &service.data);
        } 
        self.services.insert(service.id.clone(), service);
    }

    fn update_advertisement(&mut self) -> Result<(), BleError>{
        set_advertising_data(self.ble_device.get_advertising(), &mut self.advertisement())?;
        self.update_looping_data()
    }

    pub fn add_service(&mut self, service: Service) -> Result<&mut Self, BleError>{
        self.insert_service(service);
        self.update_advertisement()?;
        Ok(self)
    }

    pub fn add_services(&mut self, services: Vec<Service>) -> Result<(), BleError>{
        for service in services{
            self.insert_service(service)
        }
        self.update_advertisement()
    }

    fn reset_advertisement(&mut self) -> Result<(), BleError>{
        self.advertisement.replace(BLEAdvertisementData::new());
        self.set_name(self.advertising_name.clone());
        self.add_services(self.services.clone().into_values().collect())
    }

    // check if advertisement allows removing service
    pub fn remove_service(&mut self, service_id: &ServiceId) -> Result<(), BleError>{
        if self.services.remove(&service_id).is_none(){
            return Ok(())
        };
        self.reset_advertisement()
    }

    pub fn remove_services(&mut self, service_ids: Vec<&ServiceId>) -> Result<(), BleError>{
        let mut modified = false;
        for service_id in service_ids{
            if self.services.remove(service_id).is_some(){
                modified = true
            };
        }

        if !modified{
            return Ok(())
        }

        self.reset_advertisement()
    }
    
    /// Start advertising one particular service data 
    fn change_advertised_service_data(&mut self, service_id: &ServiceId) -> Result<(), BleError> {
        match self.services.get(service_id){
            Some(request_service) => {
                self.advertisement.borrow_mut().service_data(request_service.id.to_uuid(), &request_service.data);
                set_advertising_data(self.ble_device.get_advertising(), &mut self.advertisement())?;
                self.start()
            },
            None => Err(BleError::ServiceUnknown),
        }
    }
    
    fn stop_looping_data(&mut self)-> Result<(), BleError>{
        if self.looping_service_data{
            self.looping_service_data = false;
            self.timer_driver.remove_interrupt().map_err(BleError::TimerDriverError)?
        }
        Ok(())
    }

    pub fn advertise_service_data(&mut self, service_id: &ServiceId)-> Result<(), BleError>{
        self.stop_looping_data()?;
        self.change_advertised_service_data(service_id)
    }

    fn update_looping_data(&mut self)->Result<(), BleError>{
        if self.looping_service_data{
            self.advertise_all_service_data()?;
        }
        Ok(())
    }

    pub fn advertise_all_service_data(&mut self)-> Result<(), BleError>{
        self.looping_service_data = true;
        let services: Vec<Service> = self.services.clone().into_values().collect();
        println!("services: {:?}", services);
        let mut services = services.into_iter().cycle();
        let advertising = self.ble_device.get_advertising();
        let advertisement = self.advertisement.clone();

        let callback = move || {
            let service = services.next().unwrap();
            advertisement.borrow_mut().service_data(service.id.to_uuid(), &service.data);
            set_advertising_data(&advertising, &mut (*advertisement.borrow_mut())).unwrap();
        };

        self.timer_driver.interrupt_after_n_times(
            self.time_per_service.as_micros().try_into().unwrap_or(u64::MAX),
            None,
            true, 
            callback);
        self.timer_driver.enable().map_err(BleError::TimerDriverError)
    }

    pub fn start(&self) -> Result<(), BleError>{
        let mut ble_adv = self.ble_device.get_advertising().lock();
        ble_adv.start().map_err(|_| BleError::StartingFailure)
    }
    
    pub fn stop(&mut self) -> Result<(), BleError>{
        self.stop_looping_data()?;
        let ble_adv = self.ble_device.get_advertising().lock();
        match ble_adv.stop(){
            Ok(_) => Ok(()),
            Err(err) => match err.code(){
                esp_idf_svc::sys::BLE_HS_EALREADY => Ok(()),
                _ => Err(BleError::StoppingFailure),
            },
        }
    }
}


fn set_advertising_data(ble_adv: &Mutex<BLEAdvertising>, data: &mut BLEAdvertisementData)->Result<(), BleError>{
    let mut ble_adv = ble_adv.lock();
    loop{
        let res: Result<(), BLEError> = ble_adv.advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(data);
        if  BLEError::convert(esp_idf_svc::sys::BLE_HS_EBUSY) != res {
            return res.map_err(BleError::from);
        }
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
