use std::{ collections::HashMap, time::Duration};

use esp32_nimble::{BLEClient, BLEDevice, BLEScan};
use esp_idf_svc::hal::task::block_on;
const BLOCK: i32 = i32::MAX;
const MS_BETWEEN_SCANS: u16 = 100;

use crate::{ble::RemoteCharacteristic, utils::{auxiliary::{SharableRef, SharableRefExt}, esp32_framework_error::Esp32FrameworkError, notification::Notifier}, InterruptDriver};

use super::{BleAdvertisedDevice, BleError, BleId};

/// Driver responsible for handling the client-end of ble connections. Can be used to read, write or notify 
/// on characteristics of services of connected clients
struct _BleClient{
    ble_client: BLEClient,
    ble_scan: &'static mut BLEScan,
    connected: bool,
    time_between_scans: u16,
    notifier: Notifier,
}

#[derive(Default)]
struct BleClientUpdater{
    remote_characteristics: HashMap<BleId, RemoteCharacteristic>,
}

impl BleClientUpdater{
    fn add_characteristic(&mut self, characteristic: &RemoteCharacteristic) {
        self.remote_characteristics.insert(characteristic.id(), characteristic.duplicate());
    }
}


/// Driver responsible for handling the client-end of ble connections. Can be used to read, write or notify 
/// on characteristics of services of connected clients
#[derive(Clone)]
pub struct BleClient{
    inner: SharableRef<_BleClient>,
    updater: SharableRef<BleClientUpdater>
}

#[sharable_reference_macro::sharable_reference_wrapper]
impl _BleClient{
    /// Creates a new BleBeacon
    /// 
    /// # Arguments
    /// 
    /// - `ble_device`: A BLEDevice needed to get the BLEScan
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller]
    /// 
    /// # Returns
    /// A [_BleClient] with the default time_between_scans `TIME_BETWEEN_SCANS`, ready to connect to a ble server
    pub fn new(ble_device: & mut BLEDevice, notifier: Notifier)-> Self{
        _BleClient{ble_client: BLEClient::new(), ble_scan: ble_device.get_scan(), connected: false,time_between_scans: MS_BETWEEN_SCANS, notifier}
    }

    /// Blocking method that attempts to find a device that fullfills a condition, for a specified 
    /// ammount of time or indefinitly. Once found it will attempt to connect 
    /// 
    /// # Arguments
    /// 
    /// - `timeout`: A duration in which the client will attempt to connect to a device that fullfills the condition. If it is None then 
    /// the client will attempt to connect indefinitly
    /// - `condition`: A closure that receives a [&BleAdvertisedDevice], and returns a bool. The client will connect to any device where applying
    /// this clossure returns true. This way the client can connect to any device that advertises itslef in a certain way.
    /// 
    /// # Returns
    /// 
    /// A `Ok(())` if it was able to connect to a device that satisfice the condition before `timeout`, or a `Err(BleError)` if a connection
    /// cant be set before `timeout`.
    /// 
    /// # Errors
    /// 
    /// - `BleError::AlreadyConnected`: if already connected
    /// - `BleError::TimeOut`: if didnt find any device that meets condition in `timeout`
    /// - `BleError::DeviceNotFound`: if device that was found that meets condition, desapeared when trying to connect to it
    /// - `BleError::Code`: on other errors
    pub fn connect_to_device<C: Fn(&BleAdvertisedDevice) -> bool + Send + Sync>(&mut self, timeout: Option<Duration>, condition: C)->Result<(), BleError>{
        block_on(self.connect_to_device_async(timeout, condition))
    }

    /// Blocking method that attempts to find a device that advertises a given Service, for a specified 
    /// ammount of time or indefinitly. Once found it will attempt to connect
    /// 
    /// # Arguments
    /// 
    /// - `timeout`: A duration in which the client will attempt to connect to a device that fullfills the condition. If it is None then 
    /// the client will attempt to connect indefinitly
    /// - `service_id`: A [&BleId] that a advertising devise must advertise in order for the client to connect to it
    ///
    /// # Returns
    /// Same return type as [Self::connect_to_device]
    /// 
    /// # Errors
    /// Same erros as [Self::connect_to_device]
    pub fn connect_to_device_with_service(&mut self, timeout: Option<Duration>, service_id: &BleId)->Result<(), BleError>{
        block_on(self.connect_to_device_with_service_async(timeout, service_id))
    }

    /// /// Blocking method that attempts to find a device of a given name, for a specified 
    /// ammount of time or indefinitly. Once found it will attempt to connect
    /// 
    /// # Arguments
    /// 
    /// - `timeout`: A duration in which the client will attempt to connect to a device that fullfills the condition. If it is None then 
    /// the client will attempt to connect indefinitly
    /// - `name`: A name that a advertising devise must have in order for the client to connect to it
    ///
    /// # Returns
    /// Same return type as [Self::connect_to_device]
    /// 
    /// # Errors
    /// Same erros as [Self::connect_to_device]
    pub fn connect_to_device_of_name(&mut self, timeout: Option<Duration>, name: String)->Result<(), BleError>{
        block_on(self.connect_to_device_of_name_async(timeout, name))
    }

    /// Non blocking async version of [Self::connect_to_device]
    pub async fn connect_to_device_async<C: Fn(&BleAdvertisedDevice) -> bool + Send + Sync>(&mut self, timeout: Option<Duration>, condition: C)->Result<(), BleError>{
        self._start_scan();
        let timeout = match timeout{
            Some(timeout) => timeout.as_millis().min(i32::MAX as u128) as i32,
            None => BLOCK,
        };

        let device = self.ble_scan.find_device(timeout, |adv| {
            let adv = BleAdvertisedDevice::from(adv);
            condition(&adv)
        }).await?;

        match device{
            Some(device) => self.ble_client.connect(device.addr()).await
            .map_err(BleError::from_connection_context),
            None => Err(BleError::DeviceNotFound)
        }?;
        self.connected = true;
        Ok(())
    }

    /// Non blocking async version of [Self::connect_to_device_with_service]
    pub async fn connect_to_device_with_service_async(&mut self, timeout: Option<Duration>, service: &BleId)->Result<(), BleError>{
        self.connect_to_device_async(timeout, |adv| {
            adv.is_advertising_service(service)
        }).await
    }

    /// Non blocking async version of [Self::connect_to_device_of_name]
    pub async fn connect_to_device_of_name_async(&mut self, timeout: Option<Duration>, name: String)->Result<(), BleError>{
        self.connect_to_device_async(timeout, |adv| {
            adv.name() == name
        }).await
    }
    
    /// Blocking method that attempts to get all service ids of a given of the current connection
    ///
    /// # Returns
    /// 
    /// A `Result` containing a `Vec<BleID>` or `BleError` if a failure occured.
    /// 
    /// # Errors
    /// - `BleError::Disconnected`: if there is no connection stablished to go look for a service
    /// - `BleError::DeviceNotFound`: if the device has unexpectidly disconnected
    /// - `BleError::Code`: on other errors
    pub fn get_all_service_ids(&mut self)-> Result<Vec<BleId>, BleError>{
        block_on(self.get_all_service_ids_async())
    }

    /// Non blocking async version of [Self::get_all_service_ids]
    pub async fn get_all_service_ids_async(&mut self)-> Result<Vec<BleId>, BleError>{
        self.is_connected()?;
        let remote_services = self.ble_client.get_services().await?;
        let services = remote_services.map(|remote_service| BleId::from(remote_service.uuid())).collect();
        Ok(services)
    }
    
    /// Inner version of [TimerDriver::get_characteristic]
    async fn _get_characteristic_async(&mut self, service_id: &BleId, characteristic_id: &BleId)-> Result<RemoteCharacteristic, BleError>{
        self.is_connected()?;
        let remote_service = self.ble_client.get_service(service_id.to_uuid()).await.map_err(BleError::from_service_context)?;
        let remote_characteristic = remote_service.get_characteristic(characteristic_id.to_uuid()).await.map_err(BleError::from_characteristic_context)?;
        Ok(RemoteCharacteristic::new(remote_characteristic, self.notifier.clone()))
    }

    /// Inner version of [TimerDriver::get_all_characteristic]
    async fn _get_all_characteristics_async(&mut self, service_id: &BleId) -> Result<Vec<RemoteCharacteristic>, BleError>{
        self.is_connected()?;
        let remote_service = self.ble_client.get_service(service_id.to_uuid()).await.map_err(BleError::from_service_context)?;
        let remote_characteristics = remote_service.get_characteristics().await?.map(|remote_characteristic| {
            RemoteCharacteristic::new(remote_characteristic, self.notifier.clone())
        }).collect();
        Ok(remote_characteristics)
    }
    
    /// Sets the amount of ms for between scans
    pub fn set_time_between_scans(&mut self, ms_between_scans: u16){
        self.time_between_scans = ms_between_scans
    }

    /// Starts a scan in order to find devices to connect to
    fn _start_scan(&mut self){
        self.ble_scan.active_scan(true)
            .interval(self.time_between_scans.max(1))
            .window(self.time_between_scans.max(2) -1);
    }

    /// The conn_handle is obtained with the ConnectionInformation inside the closure of 
    /// connection_handler
    /// 
    /// # Arguments
    /// 
    /// - `min_interval`: The minimum connection interval, time between BLE events. This value 
    /// must range between 7.5ms and 4000ms in 1.25ms units, this interval will be used while transferring data
    /// in max speed.
    /// - `max_interval`: The maximum connection interval, time between BLE events. This value 
    /// must range between 7.5ms and 4000ms in 1.25ms units, this interval will be used to save energy.
    /// - `latency`: The number of packets that can be skipped (packets will be skipped only if there is no data to answer).
    /// - `timeout`: The maximum time to wait after the last packet arrived to consider connection lost. 
    /// 
    /// # Returns
    /// 
    /// A `Result` with Ok if the configuration of connection settings completed successfully, or an `BleError` if it fails.
    /// 
    /// # Errors
    /// - `BleError::Disconnected`: if there is no connection stablished to go look for a service
    /// - `BleError::DeviceNotFound`: if the device has unexpectidly disconnected
    /// - `BleError::Code`: on other errors
    pub fn set_connection_settings(&mut self, min_interval: u16, max_interval: u16, latency: u16, timeout: u16) -> Result<(), BleError>{
        self.ble_client.update_conn_params(min_interval, max_interval, latency, timeout).map_err(BleError::from_connection_params_context)
    } 

    fn is_connected(&mut self)->Result<(), BleError>{
        if !self.connected || !self.ble_client.connected(){
            return Err(BleError::Disconnected)
        }
        Ok(())
    }

    /// Disconnects the client from the current connection
    /// 
    /// # Arguments
    /// 
    /// - `service_id`: The id of the service .
    /// 
    /// # Returns
    /// 
    /// A `Result` containing `()` if it was able to disconnect or `BleError` if a failure 
    /// occured or `BleError` on failure.
    /// 
    pub fn disconnect(&mut self)-> Result<(), BleError>{
        self.connected = false;
        match self.ble_client.disconnect().map_err(BleError::from){
            Ok(_) => Ok(()),
            Err(err) => match err{
                    BleError::DeviceNotFound => Ok(()),
                    _ => Err(err)
                }
        }   
    }
}

impl BleClient{
    /// Creates a new BleBeacon
    /// 
    /// # Arguments
    /// 
    /// - `ble_device`: A BLEDevice needed to get the BLEScan
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller]
    /// 
    /// # Returns
    /// A [BleClient] with the default time_between_scans, ready to connect to a ble server
    pub fn new(ble_device: & mut BLEDevice, notifier: Notifier)-> Self{
        Self{
            inner: SharableRef::new_sharable(_BleClient::new(ble_device, notifier)),
            updater: SharableRef::new_sharable(BleClientUpdater::default())
        }
    }

    /// Blocking method that attempts to get a characteristic from a service of the current connection.
    /// 
    /// # Arguments
    /// 
    /// - `service_id`: The id of the service which owns the characteristic.
    /// - `characteristic_id`: The id of the desired characterisitc, of the given service.
    /// 
    /// # Returns
    /// 
    /// A `Result` containing a `RemoteCharacteristic` if it was able to find the characteristic in the 
    /// specified service or `BleError` if a failure occured.
    /// 
    /// # Errors
    /// - `BleError::Disconnected`: if there is no connection stablished to go look for a service
    /// - `BleError::ServiceNotFound`: if the device does not have a service of the specified id
    /// - `BleError::CharacteristicNotFound`: if the devices's service does not have a characteristic of the 
    ///    specified id
    /// - `BleError::Code`: on other errors
    pub fn get_characteristic(&mut self, service_id: &BleId, characteristic_id: &BleId)-> Result<RemoteCharacteristic, BleError>{
        block_on(self.get_characteristic_async(service_id, characteristic_id))
    }

    /// Blocking method that attempts to get all characteristics of a given service of the current connection
    /// 
    /// # Arguments
    /// 
    /// - `service_id`: The id of the service .
    /// 
    /// # Returns
    /// 
    /// A `Result` containing a `Vec<RemoteCharacteristic>` if it was able to find the specified service or 
    /// `BleError` if a failure occured.
    /// 
    /// # Errors
    /// - `BleError::Disconnected`: if there is no connection stablished to go look for a service
    /// - `BleError::ServiceNotFound`: if the device does not have a service of the specified id
    /// - `BleError::Code`: on other errors
    pub fn get_all_characteristics(&mut self, service_id: &BleId) -> Result<Vec<RemoteCharacteristic>, BleError>{
        block_on(self.get_all_characteristics_async(service_id))
    }

    /// Non blocking async version of [Self::get_characteristic]
    pub async fn get_characteristic_async(&mut self, service_id: &BleId, characteristic_id: &BleId)->Result<RemoteCharacteristic, BleError>{
        let characteristic = self.inner.deref_mut()._get_characteristic_async(service_id, characteristic_id).await?;
        self.updater.borrow_mut().add_characteristic(&characteristic);
        Ok(characteristic)
    }

    /// Non blocking async version of [Self::get_all_characteristics]
    pub async fn get_all_characteristics_async(&mut self, service_id: &BleId)->Result<Vec<RemoteCharacteristic>, BleError>{
        let characteristics = self.inner.deref_mut()._get_all_characteristics_async(service_id).await?;
        for c in &characteristics{
            self.updater.borrow_mut().add_characteristic(c);
        }
        Ok(characteristics)
    }

}

impl InterruptDriver for BleClient{
    /// Updates all characteristics that have ben gotten
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError> {
        for c in self.updater.deref_mut().remote_characteristics.values_mut(){
            c.execute_if_notified()
        }
        Ok(())
    }
}
