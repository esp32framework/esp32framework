use super::utils::{
    BleError, BleId, Characteristic, ConnectionInformation, ConnectionMode, DiscoverableMode,
    Service,
};
use crate::{
    utils::{
        auxiliary::{SharableRef, SharableRefExt},
        esp32_framework_error::Esp32FrameworkError,
        isr_queues::{ISRQueue, ISRQueueTrait},
        notification::Notifier,
    },
    InterruptDriver,
};
use esp32_nimble::{
    utilities::mutex::Mutex, BLEAdvertisementData, BLEAdvertising, BLECharacteristic, BLEDevice,
    BLEServer, BLEService, NimbleProperties,
};
use esp_idf_svc::hal::task;
use sharable_reference_macro::sharable_reference_wrapper;
use std::sync::Arc;

type ConnCallback<'a> = dyn FnMut(&mut BleServer<'a>, &ConnectionInformation) + 'a;

/// Abstraction to create a BLE server, the side that has the information to be used in a connection
/// oriented relationship. Contains:
/// * `advertising_name`: Clients scanning will see the advertising name before connection.
/// * `services`: The servere will hace information for the clients to see. All this information will be encapsulated on different services.
/// * `user_on_connection`: Callback that will be executed for each client connected.
/// * `user_on_disconnection`: Callback that will be executed for each client disconnected.
struct _BleServer<'a> {
    advertising_name: String,
    ble_server: &'a mut BLEServer,
    services: Vec<Service>,
    advertisement: &'a Mutex<BLEAdvertising>,
    user_on_connection: Option<ConnectionCallback<'a>>,
    user_on_disconnection: Option<ConnectionCallback<'a>>,
}

/// Abstraction to create a BLE server, the side that has the information to be used in a connection
/// oriented relationship. Contains:
/// * `advertising_name`: Clients scanning will see the advertising name before connection.
/// * `services`: The servere will hace information for the clients to see. All this information will be encapsulated on different services.
/// * `user_on_connection`: Callback that will be executed for each client connected.
/// * `user_on_disconnection`: Callback that will be executed for each client disconnected.
pub struct BleServer<'a> {
    inner: SharableRef<_BleServer<'a>>,
}

/// Wrapper to handle user connection and disconnections callbacks in a simpler way
struct ConnectionCallback<'a> {
    callback: Box<ConnCallback<'a>>,
    info_queue: ISRQueue<ConnectionInformation>,
    notifier: Notifier,
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
    fn new(notifier: Notifier) -> Self {
        Self {
            callback: Box::new(|_, _| {}),
            info_queue: ISRQueue::new(1000),
            notifier,
        }
    }

    /// Sets the a new callback
    ///
    /// # Arguments
    ///
    /// - `callback`: User callback to execute
    fn set_callback<C: FnMut(&mut BleServer<'a>, &ConnectionInformation) + 'a>(
        &mut self,
        callback: C,
    ) {
        self.callback = Box::new(callback);
    }

    /// Continuously tries to read from the queue to know if its time to execute the user callback
    ///
    /// # Arguments
    ///
    /// - `server`: The BleServer that is send as a parameter for the user to use in the callback
    fn handle_connection_changes(&mut self, server: &mut BleServer<'a>) {
        while let Ok(item) = self.info_queue.try_recv() {
            (self.callback)(server, &item);
        }
    }
}

#[sharable_reference_wrapper]
impl<'a> _BleServer<'a> {
    /// Creates a new _BleServer
    ///
    /// # Arguments
    ///
    /// - `name`: The name of the server
    /// - `ble_device`: A BLEDevice needed to get the BLEServer and the BLEAdvertising
    /// - `services`: A vector with multiple Service that will contain the server information
    /// - `connection_notifier`: A Notifier used to notify when the connection callback should be executed
    /// - `disconnection_notifier`: A Notifier used to notify when the disconnection callback should be executed
    ///
    /// # Returns
    ///
    /// A `Result` with the newly created _BleServer, or a `BleError` if a failure occured when setting the
    /// services
    ///
    /// # Errors
    ///
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property.
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server.
    fn new(
        name: String,
        ble_device: &mut BLEDevice,
        services: &Vec<Service>,
        connection_notifier: Notifier,
        disconnection_notifier: Notifier,
    ) -> Result<Self, BleError> {
        let mut server = _BleServer {
            advertising_name: name,
            ble_server: ble_device.get_server(),
            services: services.clone(),
            advertisement: ble_device.get_advertising(),
            user_on_connection: Some(ConnectionCallback::new(connection_notifier)),
            user_on_disconnection: Some(ConnectionCallback::new(disconnection_notifier)),
        };

        for service in services {
            server.set_service(service)?;
        }

        Ok(server)
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
    pub fn connection_handler<C: FnMut(&mut BleServer, &ConnectionInformation) + 'a>(
        &mut self,
        handler: C,
    ) -> &mut Self {
        let user_on_connection = self.user_on_connection.as_mut().unwrap();
        let notifier_ref = user_on_connection.notifier.clone();
        let mut con_info_ref = user_on_connection.info_queue.clone();
        user_on_connection.set_callback(handler);

        self.ble_server.on_connect(move |_, info| {
            notifier_ref.notify();
            _ = con_info_ref.send_timeout(
                ConnectionInformation::from_bleconn_desc(info, true, Ok(())),
                1_000_000,
            );
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
    pub fn disconnect_handler<C: FnMut(&mut BleServer, &ConnectionInformation) + 'a>(
        &mut self,
        handler: C,
    ) -> &mut Self {
        let user_on_disconnection = self.user_on_disconnection.as_mut().unwrap();
        let notifier_ref = user_on_disconnection.notifier.clone();
        let mut con_info_ref = user_on_disconnection.info_queue.clone();
        user_on_disconnection.set_callback(handler);

        self.ble_server.on_disconnect(move |info, res| {
            notifier_ref.notify();
            _ = con_info_ref.send_timeout(
                ConnectionInformation::from_bleconn_desc(info, false, res),
                1_000_000,
            );
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
    ///
    /// # Errors
    ///
    /// - `BleError::Disconnected`: if there is no connection stablished to go look for a service
    /// - `BleError::DeviceNotFound`: if the device has unexpectidly disconnected
    /// - `BleError::Code`: on other errors
    pub fn set_connection_settings(
        &mut self,
        info: &ConnectionInformation,
        min_interval: u16,
        max_interval: u16,
        latency: u16,
        timeout: u16,
    ) -> Result<(), BleError> {
        self.ble_server
            .update_conn_params(
                info.conn_handle,
                min_interval,
                max_interval,
                latency,
                timeout,
            )
            .map_err(BleError::from_connection_params_context)
    }

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
    pub fn set_advertising_interval(&mut self, min_interval: u16, max_interval: u16) -> &mut Self {
        self.advertisement
            .lock()
            .min_interval(min_interval)
            .max_interval(max_interval);
        self
    }

    /// Sets a high duty cycle has intervals between advertising packets are
    /// typically in the range of 20 ms to 100 ms.
    /// Valid only if advertisement_type is directed-connectable.
    ///
    /// # Returns
    ///
    /// The _BleServer itself
    pub fn set_high_advertising_duty_cycle(&mut self) -> &mut Self {
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
            DiscoverableMode::NonDiscoverable => {
                self.advertisement.lock().disc_mode(disc_mode.get_code())
            }
            DiscoverableMode::GeneralDiscoverable(min_interval, max_interval) => self
                .advertisement
                .lock()
                .disc_mode(disc_mode.get_code())
                .min_interval(min_interval)
                .max_interval(max_interval),
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
        self.advertisement
            .lock()
            .advertisement_type(conn_mode.get_code());
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
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property.
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server.
    pub fn set_service(&mut self, service: &Service) -> Result<(), BleError> {
        self.ble_server.create_service(service.id.to_uuid());

        for characteristic in &service.characteristics {
            self.set_characteristic(&service.id, characteristic)?;
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
    pub fn set_services(&mut self, services: &Vec<Service>) -> Result<(), BleError> {
        for service in services {
            self.set_service(service)?;
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
    pub fn set_characteristic(
        &mut self,
        service_id: &BleId,
        characteristic: &Characteristic,
    ) -> Result<(), BleError> {
        let server_service =
            task::block_on(async { self.ble_server.get_service(service_id.to_uuid()).await });

        match server_service {
            Some(service) => {
                match self.try_to_update_characteristic(service, characteristic, false) {
                    Ok(_) => Ok(()),
                    Err(_) => self.create_new_characteristic(characteristic, service),
                }
            }
            None => Err(BleError::ServiceNotFound),
        }
    }

    /// Set a new characteristic
    ///
    /// # Arguments
    ///
    /// - `service`: The service to which the characteristic will be added
    /// - `characteristic`: A Characteristic struct that will contain all the onformation of the characteristic
    ///   that wants to be set
    ///
    /// # Returns
    ///  
    /// A `Result` with Ok if the operation completed successfully, or an `BleError` if it fails.
    ///
    /// # Errors
    ///
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property
    fn create_new_characteristic(
        &self,
        characteristic: &Characteristic,
        service: &Arc<Mutex<BLEService>>,
    ) -> Result<(), BleError> {
        match NimbleProperties::from_bits(characteristic.properties.to_le()) {
            Some(properties) => {
                let charac = service
                    .lock()
                    .create_characteristic(characteristic.id.to_uuid(), properties);
                let mut unlocked_char = charac.lock();
                unlocked_char.set_value(&characteristic.data);

                for descriptor in &characteristic.descriptors {
                    match descriptor.get_properties() {
                        Ok(properties) => {
                            let ble_descriptor = unlocked_char
                                .create_descriptor(descriptor.id.to_uuid(), properties);
                            ble_descriptor.lock().set_value(&descriptor.data);
                        }
                        Err(_) => return Err(BleError::PropertiesError),
                    };
                }

                Ok(())
            }
            None => Err(BleError::PropertiesError),
        }
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
    fn try_to_update_characteristic(
        &self,
        service: &Arc<Mutex<BLEService>>,
        characteristic: &Characteristic,
        notify: bool,
    ) -> Result<(), BleError> {
        // Check if there is a characteristic with characteristic.id in this service
        let locked_service = service.lock();
        let server_characteristic = task::block_on(async {
            locked_service
                .get_characteristic(characteristic.id.to_uuid())
                .await
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
    pub fn notify_value(
        &mut self,
        service_id: &BleId,
        characteristic: &Characteristic,
    ) -> Result<(), BleError> {
        if !characteristic.is_notifiable() {
            return Err(BleError::CharacteristicNotNotifiable);
        }
        let server_service =
            task::block_on(async { self.ble_server.get_service(service_id.to_uuid()).await });
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
    pub fn start(&mut self) -> Result<(), BleError> {
        self.create_advertisement_data()?;
        self.advertisement
            .lock()
            .start()
            .map_err(|_| BleError::StartingAdvertisementError)
    }

    /// Stop the server advertisement. This function only stop the advertisement,
    /// any service running in the servil will continue running.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the stopping operation completed successfully, or a
    /// `BleError` if it fails.
    ///
    /// # Errors
    ///
    /// - `BleError::AdvertisementError`: If the advertising operation failed
    /// - `BleError::StoppingFailure`: If the stopping operation failed
    pub fn stop_advertisement(&mut self) -> Result<(), BleError> {
        self.advertisement
            .lock()
            .stop()
            .map_err(|_| BleError::StoppingFailure)
    }

    /// List all active clients.
    ///
    /// # Returns
    ///
    /// A `Vec<ConnectionInformation>` containing information about each connected client.
    pub fn list_clients(&mut self) -> Vec<ConnectionInformation> {
        self.ble_server
            .connections()
            .map(|desc| ConnectionInformation::from_bleconn_desc(&desc, true, Ok(())))
            .collect()
    }

    /// Disconnects all currently connected clients.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if all clients were successfully disconnected, or a
    /// `BleError` if it fails.
    ///
    /// # Errors
    ///
    /// - `BleError::Disconnected`: If any client fails to disconnect.
    pub fn disconnect_all_clients(&mut self) -> Result<(), BleError> {
        let clients: Vec<_> = self.ble_server.connections().collect();
        for client in clients {
            self.ble_server
                .disconnect(client.conn_handle())
                .map_err(|_| BleError::Disconnected)?;
        }
        Ok(())
    }

    /// Disconnects a specific client.
    ///
    /// # Parameters
    ///
    /// - `client`: A reference to the `ConnectionInformation` of the client to disconnect.
    /// # Returns
    ///
    /// A `Result` with Ok if all clients were successfully disconnected, or a
    /// `BleError` if it fails.
    ///
    /// # Errors
    ///
    /// - `BleError::Disconnected`: If any client fails to disconnect.
    pub fn disconnect_client(&mut self, client: &ConnectionInformation) -> Result<(), BleError> {
        self.ble_server
            .disconnect(client.conn_handle)
            .map_err(|_| BleError::Disconnected)?;

        Ok(())
    }

    /// Returns the number of currently connected BLE clients.
    ///
    /// # Returns
    ///
    /// A `usize` representing the number of clients currently connected.
    pub fn amount_of_clients(&mut self) -> usize {
        self.ble_server.connected_count()
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
    fn create_advertisement_data(&mut self) -> Result<(), BleError> {
        let mut adv_data = BLEAdvertisementData::new();
        adv_data.name(&self.advertising_name);
        for service in &self.services {
            adv_data.add_service_uuid(service.id.to_uuid());
        }
        self.advertisement
            .lock()
            .set_data(&mut adv_data)
            .map_err(|_| BleError::AdvertisementError)
    }

    /// Gets the data of a specific characteristic
    ///
    /// # Arguments
    ///
    /// - `ble_characteristic`: An `&Arc<Mutex<BLECharacteristic>>` that has the desired characteristic
    ///
    /// # Returns
    ///
    /// A `Vec<u8>` that represents the bytes of data of the characteristic
    fn read_characteristic_data(
        &self,
        ble_characteristic: &Arc<Mutex<BLECharacteristic>>,
    ) -> Vec<u8> {
        let mut ble_characteristic = ble_characteristic.lock();
        ble_characteristic.value_mut().value().to_vec()
    }

    /// Gets the data of a specific characteristic from a given service
    ///
    /// # Arguments
    ///
    /// - `service_id`: A `&BleId` that represents the id of the service that has the characteristic
    /// - `characteristic_id`: A `&BleId` that represents the id of the characteristic
    ///
    /// # Returns
    ///
    /// A `Result` containing the data if the operation is succesful, or a `BleError` if it fails
    ///
    /// # Errors
    ///
    /// - `BleError::CharacteristicNotFound`: If the characteristic is not in the indicated service
    /// - `BleError::ServiceNotFound`: If the services is not in the BleServer itself
    pub fn get_characteristic_data(
        &self,
        service_id: &BleId,
        characteristic_id: &BleId,
    ) -> Result<Vec<u8>, BleError> {
        // Get the service from the BLEServer
        let service_option =
            task::block_on(async { self.ble_server.get_service(service_id.to_uuid()).await });

        match service_option {
            Some(service_arc) => {
                // If the service is found, we get the desired characteristic
                let locked_service = service_arc.lock();
                let server_characteristic = task::block_on(async {
                    locked_service
                        .get_characteristic(characteristic_id.to_uuid())
                        .await
                });

                // Onece we have the characteristic, we get its data
                match server_characteristic {
                    Some(characteristic_arc) => {
                        Ok(self.read_characteristic_data(characteristic_arc))
                    }
                    None => Err(BleError::CharacteristicNotFound),
                }
            }
            None => Err(BleError::ServiceNotFound),
        }
    }

    /// Gets the data of all characteristics from a given service
    ///
    /// # Arguments
    ///
    /// - `service_id`: A `&BleId` that represents the id of the service that has the characteristic
    ///
    /// # Returns
    ///
    /// A `Result` containing tuples of the id and data if the operation is succesful, or a `BleError` if it fails
    ///
    /// # Errors
    ///
    /// - `BleError::ServiceNotFound`: If the services is not in the BleServer itself
    /// - `BleError::CharacteristicNotFound`: If there was an internal error with the characteristics
    pub fn get_all_service_characteristics_data(
        &self,
        service_id: &BleId,
    ) -> Result<Vec<(BleId, Vec<u8>)>, BleError> {
        let mut data = Vec::new();
        let services = self
            .services
            .iter()
            .find(|s| s.id == *service_id)
            .ok_or(BleError::ServiceNotFound)?;
        for c in &services.characteristics {
            data.push((
                c.id.clone(),
                self.get_characteristic_data(service_id, &c.id)?,
            ));
        }
        Ok(data)
    }
}

impl<'a> InterruptDriver<'a> for BleServer<'a> {
    fn update_interrupt(&mut self) -> Result<(), Esp32FrameworkError> {
        let (mut user_on_connection, mut user_on_disconnection) = self.take_connection_callbacks();
        user_on_connection.handle_connection_changes(self);
        user_on_disconnection.handle_connection_changes(self);
        self.set_connection_callbacks(user_on_connection, user_on_disconnection);
        Ok(())
    }

    fn get_updater(&self) -> Box<dyn InterruptDriver<'a> + 'a> {
        Box::new(Self {
            inner: self.inner.clone(),
        })
    }
}

impl<'a> BleServer<'a> {
    /// Creates a new _BleServer
    ///
    /// # Arguments
    ///
    /// - `name`: The name of the server
    /// - `ble_device`: A BLEDevice needed to get the BLEServer and the BLEAdvertising
    /// - `services`: A vector with multiple Service that will contain the server information
    /// - `connection_notifier`: An Notifier used to notify when the connection callback should be executed
    /// - `disconnection_notifier`: An Notifier used to notify when the disconnection callback should be executed
    ///
    /// # Returns
    ///
    /// A `Result` with the newly created _BleServer, or a `BleError` if a failure occured when setting the
    /// services
    ///
    /// # Errors
    ///
    /// - `BleError::PropertiesError`: If a characteristic on the service has an invalid property.
    /// - `BleError::ServiceNotFound`: If the service_id doesnt match with the id of a service already set on the server.
    pub(crate) fn new(
        name: String,
        ble_device: &mut BLEDevice,
        services: &Vec<Service>,
        connection_notifier: Notifier,
        disconnection_notifier: Notifier,
    ) -> Result<Self, BleError> {
        Ok(Self {
            inner: SharableRef::new_sharable(_BleServer::new(
                name,
                ble_device,
                services,
                connection_notifier,
                disconnection_notifier,
            )?),
        })
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
    fn set_connection_callbacks(
        &mut self,
        user_on_connection: ConnectionCallback<'a>,
        user_on_disconnection: ConnectionCallback<'a>,
    ) {
        self.inner.deref_mut().user_on_connection = Some(user_on_connection);
        self.inner.deref_mut().user_on_disconnection = Some(user_on_disconnection);
    }
}
