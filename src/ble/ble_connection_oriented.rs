use std::cell::RefCell;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::Arc;
use esp32_nimble::{BLEAddress, BLEService};
use esp32_nimble::{utilities::mutex::Mutex, BLEAdvertisementData, BLEAdvertising, BLEConnDesc, BLEDevice, BLEError, BLEServer, NimbleProperties};
use esp_idf_svc::hal::task;
use esp_idf_svc::hal::task::notification::Notifier;


use crate::utils::auxiliary::ISRQueue;
use crate::utils::auxiliary::SharableRef;
use crate::utils::auxiliary::SharableRefExt;
use crate::utils::esp32_framework_error::Esp32FrameworkError;
use crate::InterruptDriver;
use sharable_reference_macro::sharable_reference_wrapper;

use super::{BleError, BleId, Characteristic, ConnectionMode, Descriptor, DiscoverableMode, Service};

// TODO: How do we document this?
pub struct _BleServer<'a> {
    advertising_name: String,
    ble_server: &'a mut BLEServer,
    services: Vec<Service>,               
    advertisement: &'a Mutex<BLEAdvertising>,
    user_on_connection: Option<ConnectionCallback<'a>>,
    user_on_disconnection: Option<ConnectionCallback<'a>>
}

/// Abstraction to create a BLE server, the side that has the information to be used in a connection
/// oriented relationship. Contains:
/// * `advertising_name`: Clients scanning will see the advertising name before connection.
/// * `services`: The servere will hace information for the clients to see. All this information will be encapsulated on different services.
/// * `user_on_connection`: Callback that will be executed for each client connected.
/// * `user_on_disconnection`: Callback that will be executed for each client disconnected.
#[derive(Clone)]
pub struct BleServer<'a>{
    inner: SharableRef<_BleServer<'a>>

}

/// Wrapper to handle user connection and disconnections callbacks in a simpler way
struct ConnectionCallback<'a>{
    callback: Box<dyn FnMut(&mut BleServer<'a>, &ConnectionInformation) + 'a>,
    info_queue: ISRQueue<ConnectionInformation>,
    notifier: Arc<Notifier>
}

impl<'a> ConnectionCallback<'a> {

    /// Creates a new ConnectionCallback
    /// 
    /// # Arguments
    /// 
    /// - `notifier`: Structure to notify when the user callback needs to be executed
    /// 
    /// # Returns
    /// 
    /// A new ConnectionCallback
    fn new(notifier: Arc<Notifier>) -> Self {
        Self { callback: Box::new(|_,_| {}), info_queue: ISRQueue::new(1000), notifier}
    }

    /// Sets the a new callback
    /// 
    /// # Arguments
    /// 
    /// - `callback`: User callback to execute
    fn set_callback<C: FnMut(&mut BleServer<'a>, &ConnectionInformation) + 'a>(&mut self, callback: C){
        self.callback = Box::new(callback);
    }

    /// Continuously tries to read from the queue to know if its time to execute the user callback
    /// 
    /// # Arguments
    /// 
    /// - `server`: The BleServer that is send as a parameter for the user to use in the callback
    fn handle_connection_changes(&mut self, server: &mut BleServer<'a>){
        while let Ok(item)  = self.info_queue.try_recv() {
            (self.callback)(server, &item);
        }
    }
}

/// Contains information about the new client connected that can be user on
/// connection or disconnection callbacks.
/// 
/// - `address`: The BLEAddress of the remote device to which the server connects.
/// - `id_address`: The public or random identity BLEAddress of the connected client. This address remains constant, even if the client uses private random addresses.
/// - `conn_handle`: A unique u16 identifier for the current BLE connection. This handle is used internally by the BLE stack to manage connections.
/// - `interval`: A u16 representing the connection interval, measured in units of 1.25 ms. It determines how frequently data is exchanged between connected devices.
/// - `timeout`: A u16 representing the connection timeout, measured in units of 10 ms. If no data is received during this time, the connection is considered lost.
/// - `latency`: A u16 representing the connection latency, indicating the number of connection intervals that can be skipped by the slave if it has no data to send.
/// - `mtu`: A u16 representing the Maximum Transmission Unit, the maximum size of data that can be sent in a single transmission. This includes the payload plus protocol headers.
/// - `bonded`: A bool indicating whether the connection is bonded, meaning the devices have exchanged security keys for secure future connections.
/// - `encrypted`: A bool indicating whether the connection is encrypted, meaning the transmitted data is protected against eavesdropping.
/// - `authenticated`: A bool indicating whether the connection is authenticated, meaning an authentication process has verified the identity of the devices.
/// - `sec_key_size`: A u32 representing the size of the security key in bits used in the connection. This affects the security level of the connection.
/// - `rssi`: A Result<i8, u32> representing the received signal strength of the connection, measured in dBm. A higher negative value indicates a stronger signal. The result can be a successful i8 value or an error code u32.
/// - `disconnection_result`: A Option<u32> representing the disconnection result code, if applicable. It can be None if the connection hasn't been disconnected, or a specific error code indicating the cause of the disconnection.
#[derive(Debug, Clone, Copy)]
pub struct ConnectionInformation{
    pub address: BLEAddress,
    pub id_address: BLEAddress,
    pub conn_handle: u16,
    pub interval: u16,
    pub timeout: u16,
    pub latency: u16,
    pub mtu: u16,
    pub bonded: bool,
    pub encrypted: bool,
    pub authenticated: bool,
    pub sec_key_size: u32,
    pub rssi: Result<i8, u32>,
    pub disconnection_result: Option<u32>,
}

impl ConnectionInformation{
    /// Creates a ConnectionInformation from a BLEConnDesc
    /// 
    /// # Arguments
    /// 
    /// - `server`: The BLEConnDesc that is used to create a ConnectionInformation
    /// - `is_connected`: A boolean to know if the function was called wheter from a disconnection or a connection 
    /// - `desc_res`: Result that may contain an error if there was problem that disconnected the client
    /// 
    /// # Returns
    /// 
    /// A new ConnectionInformation
    fn from_BLEConnDesc(desc: &BLEConnDesc, is_connected: bool, desc_res: Result<(), BLEError>) -> Self {
        let mut res = None;
        if !is_connected{
            res = match desc_res {
                Ok(_) => None,
                Err(err) => Some(err.code()),
            };
        } 

        let rssi = match desc.get_rssi() {
            Ok(rssi) => Ok(rssi),
            Err(err) => Err(err.code()),
        };
        
        ConnectionInformation{
            address: desc.address(),
            id_address:desc.id_address(),
            conn_handle: desc.conn_handle(),
            interval: desc.interval(),
            timeout: desc.timeout(),
            latency: desc.latency(),
            mtu: desc.mtu(),
            bonded: desc.bonded(),
            encrypted: desc.encrypted(),
            authenticated: desc.authenticated(),
            sec_key_size: desc.sec_key_size(),
            rssi,
            disconnection_result: res,
        }
    }

}

#[sharable_reference_wrapper]
impl <'a>_BleServer<'a> {
    /// Creates a new _BleServer
    /// 
    /// # Arguments
    /// 
    /// - `name`: The name of the server
    /// - `ble_device`: A BLEDevice needed to get the BLEServer and the BLEAdvertising
    /// - `services`: A vector with multiple Service that will contain the server information
    /// - `connection_notifier`: An Arc<Notifier> used to notify when the connection callback should be executed
    /// - `disconnection_notifier`: An Arc<Notifier> used to notify when the disconnection callback should be executed
    /// 
    /// # Returns
    /// 
    /// The new created _BleServer
    pub fn new(name: String, ble_device: &mut BLEDevice, services: &Vec<Service>, connection_notifier: Arc<Notifier>, disconnection_notifier: Arc<Notifier>) -> Self {
        let mut server = _BleServer{
            advertising_name: name,
            ble_server: ble_device.get_server(),
            services: services.clone(),
            advertisement: ble_device.get_advertising(),
            user_on_connection: Some(ConnectionCallback::new(connection_notifier)),
            user_on_disconnection: Some(ConnectionCallback::new(disconnection_notifier)),
        };
            
        for service in services {
            server.set_service(service);
        }

        server
    }

    /// Sets the connection handler. The handler is a callback that will be executed when a client connects to the server
    /// 
    /// # Arguments
    /// 
    /// - `handler`: A closure thath will be executed when a client connects to the server
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    pub fn connection_handler<C: FnMut(&mut BleServer, &ConnectionInformation) + 'a>(&mut self, handler: C) -> &mut Self
    {
        let user_on_connection = self.user_on_connection.as_mut().unwrap();
        let notifier_ref = user_on_connection.notifier.clone();
        let mut con_info_ref = user_on_connection.info_queue.clone();
        user_on_connection.set_callback(handler);
        
        self.ble_server.on_connect(move |_, info| {
            unsafe{ notifier_ref.notify_and_yield(NonZeroU32::new(1).unwrap()); }
            _ = con_info_ref.send_timeout(ConnectionInformation::from_BLEConnDesc(info, true, Ok(())), 1_000_000); //
        });
        self
    }

    /// Sets the disconnection handler. The handler is a callback that will be executed when a client disconnects to the server
    /// 
    /// # Arguments
    /// 
    /// - `handler`: A closure thath will be executed when a client disconnects to the server
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    pub fn disconnect_handler<C: FnMut(&mut BleServer, &ConnectionInformation) + 'a>(&mut self, handler: C) -> &mut Self
    {
        let user_on_disconnection = self.user_on_disconnection.as_mut().unwrap();
        let notifier_ref = user_on_disconnection.notifier.clone();
        let mut con_info_ref = user_on_disconnection.info_queue.clone();
        user_on_disconnection.set_callback(handler);
        
        self.ble_server.on_disconnect(move |info, res| {
            unsafe{ notifier_ref.notify_and_yield(NonZeroU32::new(1).unwrap()); }
            _ = con_info_ref.send_timeout(ConnectionInformation::from_BLEConnDesc(info, false, res), 1_000_000);
        });
        self
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
    pub fn set_connection_settings(&mut self, info: &ConnectionInformation, min_interval: u16, max_interval: u16, latency: u16, timeout: u16) -> Result<(), BleError>{
        self.ble_server.update_conn_params(info.conn_handle, min_interval, max_interval, latency, timeout).map_err(|e| BleError::from(e)) 
    } // TODO: This func doesnt tell which errors it can return, so i can put it in the documentation

    /// Set the advertising time parameters.
    /// 
    /// # Arguments
    /// 
    /// - `min_interval`: The minimum advertising interval, time between advertisememts. This value 
    /// must range between 20ms and 10240ms in 0.625ms units.
    /// - `max_interval`: The maximum advertising intervaltime between advertisememts. TThis value 
    /// must range between 20ms and 10240ms in 0.625ms units.
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    fn set_advertising_interval(&mut self, min_interval: u16, max_interval: u16) -> &mut Self {
        self.advertisement.lock().min_interval(min_interval).max_interval(max_interval);
        self
    }

    /// Sets a high duty cycle has intervals between advertising packets are 
    /// typically in the range of 20 ms to 100 ms.
    /// Valid only if advertisement_type is directed-connectable.
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    pub fn set_high_advertising_duty_cycle(&mut self) -> &mut Self{
        self.advertisement.lock().high_duty_cycle(true);
        self
    }

    /// Sets a low duty cycle has ntervals between advertising packets are 
    /// typically in the range of 1,000 ms to 10,240 ms.
    /// Valid only if advertisement_type is directed-connectable.
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    pub fn set_low_advertising_duty_cycle(&mut self) -> &mut Self {
        self.advertisement.lock().high_duty_cycle(false);
        self
    }

    /// Sets the discoverable mode for the server.
    /// 
    /// # Arguments
    /// 
    /// - `disc_mode`: A DiscoverableMode that the user decisdes to set
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    pub fn set_discoverable_mode(&mut self, disc_mode: DiscoverableMode) -> &mut Self {
        match disc_mode {
            DiscoverableMode::NonDiscoverable => self.advertisement.lock().disc_mode(disc_mode.get_code()),
            DiscoverableMode::LimitedDiscoverable(min_interval, max_interval) => self.advertisement.lock().disc_mode(disc_mode.get_code())
                .min_interval(min_interval)
                .max_interval(max_interval),
            DiscoverableMode::GeneralDiscoverable(min_interval, max_interval) => self.advertisement.lock().disc_mode(disc_mode.get_code()).min_interval(min_interval).max_interval(max_interval),
        };
        self.advertisement.lock().disc_mode(disc_mode.get_code());
        self
    }

    ///Sets the connection mode of the advertisment.
    /// 
    /// # Arguments
    /// 
    /// - `conn_mode`: A ConnectionMode that the user decisdes to set
    /// 
    /// # Returns
    /// 
    /// The _BleServer itself
    pub fn set_connection_mode(&mut self, conn_mode: ConnectionMode) -> &mut Self {
        self.advertisement.lock().advertisement_type(conn_mode.get_code());
        self
    }
    
    /// Sets or overwrites a service to the server.
    /// 
    /// # Arguments
    /// 
    /// - `service`: A Service struct
    /// 
    /// # Returns
    /// 
    /// A `Result` with Ok if the operation completed successfully, or an `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property
    pub fn set_service(&mut self, service: &Service) -> Result<(), BleError> {
        self.ble_server.create_service(service.id.to_uuid());

        for characteristic in &service.characteristics{
            self.set_characteristic(service.id.clone(), characteristic)?;
        }
        Ok(())
    }

    /// Sets or overwrites multiple services to the server.
    /// 
    /// # Arguments
    /// 
    /// - `services`: A vector with multiple Services to set
    /// 
    /// # Returns
    /// 
    /// A `Result` with Ok if the operation completed successfully, or an `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property
    pub fn set_services(&mut self, services: &Vec<Service>) -> Result<(),BleError> {
        for service in services {
            self.set_service(&service)?;
        }
        Ok(())
    }

    /// Set a new characteristic or update the value in an existent characteristic to the server.
    /// 
    /// # Arguments
    /// 
    /// - `service_id`: A BleId to identify the service the charactersitic is part of.
    /// - `characteristic`: A Characteristic struct that will contain all the onformation of the characteristic that wants to be set
    /// 
    /// # Returns
    ///  
    /// A `Result` with Ok if the operation completed successfully, or an `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server
    pub fn set_characteristic(&mut self, service_id: BleId, characteristic: &Characteristic) -> Result<(), BleError> {
        let server_service = task::block_on(async {
            self.ble_server.get_service(service_id.to_uuid()).await
        });

        // Check if there is a service with 'service_id' as its id.
        if let Some(service) = server_service {

            match self.try_to_update_characteristic(service, characteristic, false) {
                Ok(_) => return Ok(()),
                Err(_) => {
                    // Create a new characteristic
                    match NimbleProperties::from_bits(characteristic.properties.to_le()) {
                        Some(properties) => {

                            let charac = service.lock(). create_characteristic(
                                characteristic.id.to_uuid(),
                                properties,
                            );
                            let mut unlocked_char = charac.lock();
                            unlocked_char.set_value(&characteristic.data);

                            for descriptor in &characteristic.descriptors {
                                match descriptor.get_properties() {
                                    Ok(properties) => {
                                        let ble_descriptor = unlocked_char.create_descriptor(descriptor.id.to_uuid(), properties);
                                        ble_descriptor.lock().set_value(&descriptor.data);
                                    },
                                    Err(_) => {return Err(BleError::PropertiesError)},
                                };
                            }

                            return Ok(());
                        },
                        None => {return Err(BleError::PropertiesError)},
                    }
                }
            }
        }
        Err(BleError::ServiceNotFound)
    }

    /// Checks if there is a BLECharacteristic on the BLEService with the corresponding id. If it exists, it updates its value. Apart from that,
    /// depending on the notify boolean parameter, it may notify the connected device of the changed value.
    /// 
    /// # Arguments
    /// 
    /// - `service`: An Arc<Mutex<BLEService>> that contains the service that has the characteristic to update
    /// - `characteristic`: A Characteristic struct that contains the updated information
    /// - `notify`: A boolean that indicates wheter to notify the characteristic or not.
    /// 
    /// # Returns
    /// 
    /// A `Result` with Ok if the update operation completed successfully, or an `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::CharacteristicNotFound`: If the characteristic was not setted before on the server
    fn try_to_update_characteristic(&self, service: &Arc<Mutex<BLEService>>, characteristic: &Characteristic, notify: bool) -> Result<(), BleError> {
        // Check if there is a characteristic with characteristic.id in this service
        let locked_service = service.lock();
        let server_characteristic = task::block_on(async {
            locked_service.get_characteristic(characteristic.id.to_uuid()).await
        });

        if let Some(server_characteristic) = server_characteristic {
            let mut res_characteristic = server_characteristic.lock();
            res_characteristic.set_value(&characteristic.data);
            if notify {
                res_characteristic.notify();
            }
            return Ok(());
        }
        
        Err(BleError::CharacteristicNotFound)
    }
    
    /// Notifies to the client the value of the characteristic
    /// 
    /// # Arguments
    /// 
    /// - `service_id`: A BleId to identify the service the charactersitic is part of.
    /// - `charactersitic`: A Characteristic struct that represents the characteristic to notify.
    /// 
    /// # Returns 
    /// 
    /// A `Result` with Ok if the notify operation completed successfully, or an `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server
    /// - `BleError::CharacteristicNotFound`: If the characteristic was not setted before on the server
    pub fn notify_value(&mut self, service_id: BleId, characteristic: &Characteristic) -> Result<(), BleError> {
        let server_service = task::block_on(async {
            self.ble_server.get_service(service_id.to_uuid()).await
        });

        if let Some(service) = server_service {
            self.try_to_update_characteristic(service, characteristic, true)?;
            return Ok(());
        }
        Err(BleError::ServiceNotFound)
        
    }

    /// Starts the server and its advertisement
    /// 
    /// # Returns
    /// 
    /// A `Result` with Ok if the starting operation completed successfully, or a `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::AdvertisementError`: If the advertising operation failed
    /// - `BleError::StartingAdvertisementError`: If the starting operation failed
    pub fn start(&mut self) -> Result<(), BleError>{
        self.create_advertisement_data()?;
        self.advertisement.lock().start().map_err(|_| BleError::StartingAdvertisementError)
    }

    /// Creates the necessary advertisement data with the user settings
    /// 
    /// # Returns 
    /// 
    /// A `Result` with Ok if the create operation completed successfully, or a `BleError` if it fails.
    /// 
    /// # Errors
    /// 
    /// - `BleError::AdvertisementError`: If the advertising operation failed
    fn create_advertisement_data(&mut self) -> Result<(), BleError>{
        let mut adv_data = BLEAdvertisementData::new();
        adv_data.name(&self.advertising_name);
        for service in &self.services {
            adv_data.add_service_uuid(service.id.to_uuid());
        }
        self.advertisement.lock().set_data(&mut adv_data).map_err(|_| BleError::AdvertisementError)
    }
}

// TODO: refactor this!
impl<'a> InterruptDriver for BleServer<'a>{
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError> {
        let (mut user_on_connection, mut user_on_disconnection) = self.take_connection_callbacks();
        user_on_connection.handle_connection_changes(self);
        user_on_disconnection.handle_connection_changes(self);
        self.set_connection_callbacks(user_on_connection, user_on_disconnection);
        Ok(())
    }
}

impl<'a> BleServer<'a>{

    /// Creates a new BleServer
    /// 
    /// # Arguments
    /// 
    /// - `name`: The name the server will use
    /// - `ble_device`: A BLEDevice needed to get the BLEServer and the BLEAdvertising
    /// - `services`: A vector with multiple Service that will contain the server information
    /// - `connection_notifier`: An Arc<Notifier> used to notify when the connection callback should be executed
    /// - `disconnection_notifier`: An Arc<Notifier> used to notify when the disconnection callback should be executed
    /// 
    /// # Returns
    /// 
    /// The new created BleServer
    pub fn new(name: String, ble_device: &mut BLEDevice, services: &Vec<Service>, connection_notifier: Arc<Notifier>, disconnection_notifier: Arc<Notifier>) -> Self {
        Self { inner: SharableRef::new_sharable(
            _BleServer::new(name, ble_device, services, connection_notifier, disconnection_notifier)
        ) }
    }

    /// Takes ownership of both of the connection and disconnection callbacks
    /// 
    /// # Returns
    /// 
    /// A tuple containing the connection and disconnection callbacks
    fn take_connection_callbacks(&mut self) -> (ConnectionCallback<'a>, ConnectionCallback<'a>) {
        let user_on_connection = self.inner.deref_mut().user_on_connection.take().unwrap();
        let user_on_disconnection = self.inner.deref_mut().user_on_disconnection.take().unwrap();
        (user_on_connection, user_on_disconnection)
    }

    /// Sets both the connection and disconnection callbacks
    /// 
    /// # Arguments
    /// 
    /// - `user_on_connection`: A ConnectionCallback containing everything that is needed for the connection callback handling
    /// - `user_on_disconnection`: A ConnectionCallback containing everything that is needed for the disconnection callback handling
    fn set_connection_callbacks(&mut self, user_on_connection: ConnectionCallback<'a>, user_on_disconnection: ConnectionCallback<'a>){
        self.inner.deref_mut().user_on_connection = Some(user_on_connection);
        self.inner.deref_mut().user_on_disconnection = Some(user_on_disconnection);
    }
}
