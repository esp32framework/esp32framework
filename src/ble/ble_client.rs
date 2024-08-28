use std::{ time::Duration};

use esp32_nimble::{utilities::BleUuid, BLEClient, BLEDevice, BLEScan};
use esp_idf_svc::hal::{ task::block_on};
const BLOCK: i32 = i32::MAX;

use super::{BleError, BleId};

pub struct BleClient{
    ble_client: BLEClient,
    ble_scan: &'static mut BLEScan,
    time_between_scans: u16
}

impl BleClient{
    pub fn new(ble_device: & mut BLEDevice)-> Self{
        BleClient{ble_client: BLEClient::new(), ble_scan: ble_device.get_scan(), time_between_scans: 100}
    }

    async fn connect_to_uuid(&mut self, id: BleId, timeout: Option<Duration>){
        //let scan = self.start_scan();
        
    }

    pub fn connect_to_any_with_service(&mut self, service: BleId, timeout: Option<Duration>)->Result<(), BleError>{
        block_on(self.connect_to_any_with_service_async(service, timeout))
    }

    pub async fn connect_to_any_with_service_async(&mut self, service: BleId, timeout: Option<Duration>)->Result<(), BleError>{
        self._start_scan();
        let timeout = match timeout{
            Some(timeout) => timeout.subsec_millis() as i32,
            None => BLOCK as i32,
        };
        
        let service = service.to_uuid();
        let device = self.ble_scan.find_device(timeout, |adv| {
            println!("Encontre a uno con el service: {:?}", adv.get_service_uuids());
            adv.get_service_uuids().find(|adv_service| **adv_service == service).is_some()
        }).await.map_err(BleError::from)?;
        
        match device{
            Some(device) => self.ble_client.connect(device.addr()).await
                .map_err(BleError::from) , // no lo encuentra, ya esta conectado
            None => Err(BleError::DeviceNotFound)
        }
    }
    // closure que devuelva true si se quiere conectar a uno en particular
    fn connect_to_any(&self ){
        todo!()
    }

    fn _start_scan(&mut self){
        self.ble_scan.active_scan(true)
            .interval(self.time_between_scans.max(1))
            .window(self.time_between_scans.max(2) -1);
    }
    
    fn get_characteristic(service_id: BleUuid){

    }

    fn get_all_characteristics(service_id: BleUuid){

    }

    fn get_service(){

    }

    fn get_all_services(){
        
    }

    fn update_connection_params(){
        todo!()
    }
    
    fn disconnect(){
        todo!()
    }
}