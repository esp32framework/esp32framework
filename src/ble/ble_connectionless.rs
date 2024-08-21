use esp32_nimble::{utilities::{mutex::Mutex, BleUuid}, BLEAdvertisementData, BLEAdvertising, BLEDevice, BLEError};
use std::{cell::RefCell, collections::HashMap, hash::Hash, rc::Rc, time::Duration};
use super::{Service, BleId, BleError};
use crate::utils::{auxiliary::{SharableRef, SharableRefExt}, timer_driver::{TimerDriver, TimerDriverError}};

pub struct BleBeacon<'a>{
    advertising_name: String,
    ble_device: &'a mut BLEDevice,
    services: SharableRef<HashMap<BleId,Service>>,
    advertisement: SharableRef<BLEAdvertisementData>,
    timer_driver: TimerDriver<'a>,
    time_per_service: Duration,
}

impl <'a>BleBeacon<'a>{
    pub fn new(ble_device: &'a mut BLEDevice, timer_driver: TimerDriver<'a>, advertising_name: String) -> Result<Self, BleError>{
        let mut advertisement = BLEAdvertisementData::new();
        advertisement.name(&advertising_name);
        Ok(BleBeacon{
            advertising_name, 
            ble_device, 
            services: SharableRef::new_sharable(HashMap::new()),
            advertisement: Rc::new(RefCell::from(advertisement)),
            timer_driver,
            time_per_service: Duration::from_secs(1),
        })
    }

    /// Sets the name of the beacon
    pub fn set_name(&mut self, name: String) -> &mut Self{
        self.advertisement.deref_mut().name(name.as_str());
        self.advertising_name = name;
        self
    }
    
    /// Adds the service to the advertisement and the services. If service was already inserted then 
    /// only sets the service data in the advertisement.
    fn insert_service(&mut self, service: &Service){
        add_service_to_advertising(&mut self.advertisement.deref_mut(), service, self.services.deref().contains_key(&service.id));
        self.services.deref_mut().insert(service.id.clone(), service.clone());
    }
    
    fn update_advertisement(&mut self) -> Result<(), BleError>{
        set_advertising_data(self.ble_device.get_advertising(), &mut self.advertisement.deref_mut())
    }
    
    /// Adds a service to the beacon which can be advertised. If Service is already set, then the 
    /// service data is changed
    pub fn set_service(&mut self, service: &Service) -> Result<&mut Self, BleError>{
        self.insert_service(service);
        self.update_advertisement()?;
        Ok(self)
    }
    
    /// Adds services to the beacon which can be advertised. If a Service is already set, then the 
    /// service data is changed
    pub fn set_services(&mut self, services: &Vec<Service>) -> Result<(), BleError>{
        for service in services{
            self.insert_service(service)
        }
        self.update_advertisement()
    }
    
    /// Resets the advertisement using beacon name and services
    fn reset_advertisement(&mut self) -> Result<(), BleError>{
        let mut advertisement = BLEAdvertisementData::new();
        for service in self.services.deref().values(){
            add_service_to_advertising(&mut advertisement, service, false);
        }
        self.advertisement.replace(advertisement);
        self.set_name(self.advertising_name.clone());
        self.update_advertisement()
    }

    /// Removes the specified service from the beacon
    pub fn remove_service(&mut self, service_id: &BleId) -> Result<&mut Self, BleError>{
        self.services.deref_mut().remove(service_id);
        self.reset_advertisement()?;
        Ok(self)
    }
    
    /// Removes the specified servicea from the beacon
    pub fn remove_services(&mut self, service_ids: &Vec<BleId>)->Result<(), BleError>{
        for service_id in service_ids{
            self.services.deref_mut().remove(service_id);
        }
        self.reset_advertisement()
    }
    
    /// Start advertising one particular service data 
    fn change_advertised_service_data(&mut self, service_id: &BleId) -> Result<(), BleError> {
        match self.services.deref().get(service_id){
            Some(request_service) => {
                self.advertisement.borrow_mut().service_data(request_service.id.to_uuid(), &request_service.data);
                set_advertising_data(self.ble_device.get_advertising(), &mut self.advertisement.deref_mut())?;
                self.start()
            },
            None => Err(BleError::ServiceUnknown),
        }
    }
    
    fn stop_looping_data(&mut self)-> Result<(), BleError>{
        self.timer_driver.remove_interrupt().map_err(BleError::TimerDriverError)
    }

    /// Set the beacon to advertise the data of a specified service. If beacon was looping data then 
    /// it stops.
    pub fn advertise_service_data(&mut self, service_id: &BleId)-> Result<(), BleError>{
        self.stop_looping_data()?;
        self.change_advertised_service_data(service_id)
    }

    /// Sets the time the beacon will advertise the data of a service if [`advertise_all_service_data`]
    /// was called 
    pub fn set_time_per_service(&mut self, dur: Duration){
        self.time_per_service = dur
    }

    /// The beacon advertises the data of each service every fixed duration. If services are added or 
    /// removed this is reflected. The time per service can be set with []
    pub fn advertise_all_service_data(&mut self)-> Result<(), BleError>{
        let services = self.services.clone();
        let advertising = self.ble_device.get_advertising();
        let advertisement = self.advertisement.clone();
        let mut i = 0;

        let callback = move || {
            let services = services.deref();
            i = i % services.len();
            let service = services.values().collect::<Vec<&Service>>()[i];
            advertisement.borrow_mut().service_data(service.id.to_uuid(), &service.data);
            set_advertising_data(&advertising, &mut (*advertisement.borrow_mut())).unwrap();
            i+=1
        };

        self.timer_driver.interrupt_after_n_times(
            self.time_per_service.as_micros().try_into().unwrap_or(u64::MAX),
            None,
            true, 
            callback);
        self.timer_driver.enable().map_err(BleError::TimerDriverError)
    }

    /// Start advertising set services of the beacon
    pub fn start(&self) -> Result<(), BleError>{
        let mut ble_adv = self.ble_device.get_advertising().lock();
        ble_adv.start().map_err(|_| BleError::StartingFailure)
    }
    
    /// Stop advertising set services of the beacon
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

fn add_service_to_advertising(data: &mut BLEAdvertisementData, service: &Service, only_data: bool){
    if !only_data{
        data.add_service_uuid(service.id.to_uuid());
    }
    if !service.data.is_empty(){
        data.service_data(service.id.to_uuid(), &service.data);
    } 
}