use std::{ time::Duration};

use esp32_nimble::{utilities::BleUuid, BLEClient, BLEDevice, BLEScan};
use esp_idf_svc::hal::{ task::block_on};
const BLOCK: i32 = i32::MAX;

use super::{BleAdvertisedDevice, BleError, BleId};

pub struct BleClient{
    ble_client: BLEClient,
    ble_scan: &'static mut BLEScan,
    time_between_scans: u16
}

impl BleClient{
    pub fn new(ble_device: & mut BLEDevice)-> Self{
        BleClient{ble_client: BLEClient::new(), ble_scan: ble_device.get_scan(), time_between_scans: 100}
    }

    pub fn prueba(&mut self){
        block_on(
            self.prueba_async()
        )
    }

    async fn prueba_async(&mut self){
        let services = self.ble_client.get_services().await.unwrap();
        for s in services{
            println!("service: {}", s.to_string());
            for c in s.get_characteristics().await.unwrap(){
                println!("char: {}", c.uuid());
                if c.uuid().to_string() == "01010101-0000-1000-8000-00805f9b34fb"{
                    println!("Read_value {:?}", c.read_value().await);

                }
                //if c.can_read() && !c.can_write(){
                //    println!("Read_value {:?}", c.read_value().await);
                //}

            }
        }
    }

    pub async fn connect_to_device_async<C: Fn(&BleAdvertisedDevice) -> bool + Send + Sync>(&mut self, timeout: Option<Duration>, condition: C)->Result<(), BleError>{
        self._start_scan();
        let timeout = match timeout{
            Some(timeout) => timeout.subsec_millis() as i32,
            None => BLOCK as i32,
        };
        
        let device = self.ble_scan.find_device(timeout, |adv| {
            let adv = BleAdvertisedDevice::from(adv);
            condition(&adv)
        }).await.map_err(BleError::from)?;
        
        match device{
            Some(device) => self.ble_client.connect(device.addr()).await
            .map_err(BleError::from) , // no lo encuentra, ya esta conectado
            None => Err(BleError::DeviceNotFound)
        }
    }
    
    pub async fn connect_to_device_with_service_async(&mut self, timeout: Option<Duration>, service: &BleId)->Result<(), BleError>{
        self.connect_to_device_async(timeout, |adv| {
            println!("EL que publicta tiene los servicios: {:?}", adv.get_service_uuids());
            adv.is_advertising_service(service)
        }).await
    }

    pub async fn connect_to_device_of_name_async(&mut self, timeout: Option<Duration>, name: String)->Result<(), BleError>{
        self.connect_to_device_async(timeout, |adv| {
            println!("El que publica tiene el nombre: {}", adv.name());
            adv.name() == name
        }).await
    }

    pub fn connect_to_device<C: Fn(&BleAdvertisedDevice) -> bool + Send + Sync>(&mut self, timeout: Option<Duration>, condition: C)->Result<(), BleError>{
        block_on(self.connect_to_device_async(timeout, condition))
    }

    pub fn connect_to_device_with_service(&mut self, timeout: Option<Duration>, service: &BleId)->Result<(), BleError>{
        block_on(self.connect_to_device_with_service_async(timeout, service))
    }

    pub fn connect_to_device_of_name(&mut self, timeout: Option<Duration>, name: String)->Result<(), BleError>{
        block_on(self.connect_to_device_of_name_async(timeout, name))
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