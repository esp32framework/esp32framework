use std::{ time::Duration};

use esp32_nimble::{utilities::BleUuid, BLEClient, BLEDevice, BLERemoteCharacteristic, BLEScan};
use esp_idf_svc::hal::{ delay::FreeRtos, task::block_on};
const BLOCK: i32 = i32::MAX;

use crate::ble::RemoteCharacteristic;

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
        
        let services = block_on(
            self.prueba_async()
        );

        println!("Services {:?}", services);
    }

    async fn prueba_async(&mut self){
        let service = self.ble_client.get_service(BleUuid::Uuid32(0x12345678)).await.unwrap();
        loop{
            let mut characteristics: Vec<&mut BLERemoteCharacteristic> = service.get_characteristics().await.unwrap().collect();
            for c in &mut characteristics{
                println!("char: {}", c.uuid());
                if c.can_read(){
                    println!("Read_value {:?}", c.read_value().await);
                }

                println!("\n\n")
            }
            println!(")===================================(");
            FreeRtos::delay_ms(1000)
        }
        /*
        let services = self.ble_client.get_services().await.unwrap();
        for s in services{
            println!("service: {}", s.to_string());
            for c in s.get_characteristics().await.unwrap(){
                println!("char: {}", c.uuid());
                if c.can_read(){
                    println!("Readable: {}", c.can_read());
                    println!("Read_value {:?}", c.read_value().await);

                }
                FreeRtos::delay_ms(1000)
                //if c.can_read() && !c.can_write(){
                //    println!("Read_value {:?}", c.read_value().await);
                //}

            }
        }
        */
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

    pub fn get_characteristic(&mut self, service_id: &BleId, characteristic_id: &BleId)-> Result<RemoteCharacteristic, BleError>{
        block_on(self.get_characteristic_async(service_id, characteristic_id))
    }
    
    
    pub async fn get_characteristic_async(&mut self, service_id: &BleId, characteristic_id: &BleId)-> Result<RemoteCharacteristic, BleError>{
        let remote_service = self.ble_client.get_service(service_id.to_uuid()).await.unwrap();
        let remote_characteristic = remote_service.get_characteristic(characteristic_id.to_uuid()).await.unwrap();
        Ok(RemoteCharacteristic::from(remote_characteristic))
    }
    
    pub async fn get_all_characteristics_async(&mut self, service_id: &BleId) -> Result<Vec<RemoteCharacteristic>, BleError>{
        let remote_service = self.ble_client.get_service(service_id.to_uuid()).await.unwrap();
        let remote_characteristics = remote_service.get_characteristics().await.unwrap().map(|remote_characteristic|
            RemoteCharacteristic::from(remote_characteristic)
        ).collect();
        Ok(remote_characteristics)
    }
    
    pub fn get_all_characteristics(&mut self, service_id: &BleId) -> Result<Vec<RemoteCharacteristic>, BleError>{
        block_on(self.get_all_characteristics_async(service_id))

    }

    pub async fn get_all_services(&mut self)-> Result<Vec<BleId>, BleError>{
        let remote_services = self.ble_client.get_services().await.map_err(BleError::from)?;
        let services = remote_services.map(|remote_service| BleId::from(remote_service.uuid())).collect();
        Ok(services)
        
    }

    fn update_connection_params(){
        todo!()
    }
    
    fn disconnect(){
        todo!()
    }
}