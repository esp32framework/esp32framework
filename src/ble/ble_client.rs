use std::{ collections::HashMap, ops::Rem, time::Duration};

use esp32_nimble::{utilities::BleUuid, BLEClient, BLEDevice, BLERemoteCharacteristic, BLEScan};
use esp_idf_svc::hal::{ delay::FreeRtos, task::block_on};
const BLOCK: i32 = i32::MAX;

use crate::{ble::RemoteCharacteristic, utils::{auxiliary::{SharableRef, SharableRefExt}, esp32_framework_error::Esp32FrameworkError, notification::Notifier}, InterruptDriver};

use super::{BleAdvertisedDevice, BleError, BleId};

pub struct _BleClient{
    ble_client: BLEClient,
    ble_scan: &'static mut BLEScan,
    time_between_scans: u16,
    notifier: Notifier,
    remote_characteristics: HashMap<BleId, RemoteCharacteristic>,
}

#[derive(Clone)]
pub struct BleClient{
    inner: SharableRef<_BleClient>
}

#[sharable_reference_macro::sharable_reference_wrapper]
impl _BleClient{
    pub fn new(ble_device: & mut BLEDevice, notifier: Notifier)-> Self{
        _BleClient{ble_client: BLEClient::new(), ble_scan: ble_device.get_scan(), time_between_scans: 100, notifier, remote_characteristics: HashMap::new()}
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
        let remote_characteristic = RemoteCharacteristic::new(remote_characteristic, self.notifier.clone());
        self.remote_characteristics.insert(remote_characteristic.id(), remote_characteristic.duplicate());
        Ok(remote_characteristic)
    }
    
    pub async fn get_all_characteristics_async<'a>(&mut self, service_id: &BleId) -> Result<Vec<RemoteCharacteristic>, BleError>{
        let remote_service = self.ble_client.get_service(service_id.to_uuid()).await.unwrap();
        let remote_characteristics = remote_service.get_characteristics().await.unwrap().map(|remote_characteristic| {
            let remote_characteristic = RemoteCharacteristic::new(remote_characteristic, self.notifier.clone());
            self.remote_characteristics.insert(remote_characteristic.id(), remote_characteristic.duplicate());
            remote_characteristic
        }).collect();
        Ok(remote_characteristics)
    }
    
    pub fn get_all_characteristics<'a>(&mut self, service_id: &BleId) -> Result<Vec<RemoteCharacteristic>, BleError>{
        block_on(self.get_all_characteristics_async(service_id))
    }

    pub fn get_all_service_ids(&mut self)-> Result<Vec<BleId>, BleError>{
        block_on(self.get_all_service_ids_async())
    }

    pub async fn get_all_service_ids_async(&mut self)-> Result<Vec<BleId>, BleError>{
        let remote_services = self.ble_client.get_services().await.map_err(BleError::from)?;
        let services = remote_services.map(|remote_service| BleId::from(remote_service.uuid())).collect();
        Ok(services)
    }

    //TODO lo ponemos?
    fn update_connection_params(){
        todo!()
    }
    
    pub fn disconnect(&mut self)-> Result<(), BleError>{
        self.ble_client.disconnect().map_err(BleError::from)
    }
}

impl BleClient{
    pub fn new(ble_device: & mut BLEDevice, notifier: Notifier)-> Self{
        Self { inner: SharableRef::new_sharable(_BleClient::new(ble_device, notifier)) }
    }
}

#[sharable_reference_macro::sharable_reference_wrapper]
impl InterruptDriver for _BleClient{
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError> {
        for c in self.remote_characteristics.values_mut(){
            c.update_interrupt()
        }
        Ok(())
    }
}