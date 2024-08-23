use std::num::NonZeroU32;
use std::{collections::HashMap, sync::Arc};
use std::sync::Mutex as Mutex2;

use esp32_nimble::{utilities::mutex::Mutex, uuid128, BLEAdvertisementData, BLEAdvertising, BLEConnDesc, BLEDevice, BLEError, BLEServer, NimbleProperties};
use esp_idf_svc::hal::task;
use esp_idf_svc::hal::task::notification::Notifier;

use crate::utils::esp32_framework_error::Esp32FrameworkError;
use crate::InterruptDriver;

use super::{BleError, BleId, Characteristic, Service};

pub struct BleServer<'a> {
    advertising_name: String,
    ble_server: &'a mut BLEServer,
    services: Vec<Service>, 
    advertisement: &'a Mutex<BLEAdvertising>,
    user_on_connection: ConnectionCallback<'a>
}

struct ConnectionCallback<'a>{
    callback: Option<Box<dyn FnMut(&mut BleServer<'a>, ConnectionInformation) + 'a>>,
    con_info: Arc<Mutex2<Vec<ConnectionInformation>>>,
    notifier: Arc<Notifier>
}

impl<'a> ConnectionCallback<'a>{
    fn new(notifier: Arc<Notifier>) -> Self{
        Self { callback: None, con_info: Arc::new(Mutex2::from(Vec::new())), notifier}
    }
}

#[derive(Debug, Clone)]
struct ConnectionInformation{
    address: String,
    id_address: String,
    conn_handle: u16,
    interval: u16,
    timeout: u16,
    latency: u16,
    mtu: u16,
    bonded: bool,
    encrypted: bool,
    authenticated: bool,
    sec_key_size: u32,
    rssi: Result<i8, BLEError>
}

impl ConnectionInformation{
    pub fn from_BLEConnDesc(desc: &BLEConnDesc) -> Self{
        ConnectionInformation{
            address: desc.address().to_string(),
            id_address: desc.id_address().to_string(),
            conn_handle: desc.conn_handle(),
            interval: desc.interval(),
            timeout: desc.timeout(),
            latency: desc.latency(),
            mtu: desc.mtu(),
            bonded: desc.bonded(),
            encrypted: desc.encrypted(),
            authenticated: desc.authenticated(),
            sec_key_size: desc.sec_key_size(),
            rssi: desc.get_rssi(),
        }
    }
}

impl <'a>BleServer<'a> {
    pub fn new(name: String, ble_device: &mut BLEDevice, services: Vec<Service>, notifier: Arc<Notifier>) -> Self {
        let mut server = BleServer{
            advertising_name: name,
            ble_server: ble_device.get_server(),
            services: services.clone(),
            advertisement: ble_device.get_advertising(),
            user_on_connection: ConnectionCallback::new(notifier),
            
        };
            
        for service in  &services {
            server.set_service(service);
        }

        server
    }

/*
/// Modify to handle operations like:
///     Enable/disable services (max clients or specific client)
///     Notify new user with some data

  pub fn connection_handler_with_information<C>(&mut self, mut handler: C) -> &mut Self
    where
        C: FnMut(&ConnectionInformation) + Send + Sync + 'static,
    {
        self.ble_server.on_connect(move |_, info| {
            handler(info);
        });
        self
    }

    
    connection_handler_and_enable_services(&mut self, mut handler: C, services: Vec<Service>) {
        self.ble_server.on_connect(move |server, info| {
            handler(info);
            server.set_service(Service)
            for service in services{
                let server_service = server.create_service(service.id.to_uuid());
        
                for characteristic in &service.characteristics{
                    match NimbleProperties::from_bits(characteristic.properties.to_le()) {
                    Some(properties) => {
                        let mut charac = server_service.lock().create_characteristic(
                            characteristic.id.to_uuid(),
                            properties,
                        );
                        charac.lock().set_value(&characteristic.data);
                    },
                    None => {},
                }
                }
            }
        });
        self
    }

// connection_handler_and_disable_services

// connection_handler_and_notify
*/

    pub fn connection_handler<C: FnMut(&mut Self, &ConnectionInformation) + Send + Sync + 'a>(&mut self, mut handler: C) -> &mut Self
    {
        let notifier_ref = self.user_on_connection.notifier.clone();
        let con_info_ref = self.user_on_connection.con_info.clone();
        
        self.ble_server.on_connect(move |_, info| {
            con_info_ref.lock().unwrap().push(ConnectionInformation::from_BLEConnDesc(info));
            unsafe{ notifier_ref.notify_and_yield(NonZeroU32::new(1).unwrap()); }
        });
        self
    }

    /// Ver que error pueden salir al desconectarse para ver si se pueden mapear a los nuestros
    /// En caso de poder hacerlo agregar el Result al closure del usuario
    ///    let handler = |desc: &BLEConnDesc| {
    ///     println!("Desconexión detectada con descriptor: {:?}", desc);
    /// };

    /// // Llama a `disconnect_handler` pasando el `handler` del usuario.
    /// my_struct.disconnect_handler(handler);
    pub fn disconnect_handler<H>(&mut self, mut handler: H) -> &mut Self
    where
        H: FnMut(&ConnectionInformation) + Send + Sync + 'static,
    {
        // Convertir el handler del usuario en un callback con la firma requerida.
        let closure: Box<dyn FnMut(&ConnectionInformation, Result<(), BLEError>) + Send + Sync> = Box::new(move |desc, _result| {
            handler(desc);
        });

        self.ble_server.on_disconnect(closure);
        self
    }

    // CHANGE THIS!
    pub fn set_connection_settings(){
        todo!()
    }

    /// Set or overwrite a service to the server
    pub fn set_service(&mut self, service: &Service) -> Result<(),BleError> {
        self.ble_server.create_service(service.id.to_uuid());

        for characteristic in &service.characteristics{
            self.set_characteristic(service.id.clone(), characteristic)?;
        }
        Ok(())
    }

    pub fn set_services(&mut self, services: Vec<Service>) -> Result<(),BleError> {
        for service in services {
            self.set_service(&service)?;
        }
        Ok(())
    }

    /// Set a characteristic to the server. Returns an error if the service does not exist or the properties are invalid
    pub fn set_characteristic(&mut self, service_id: BleId, characteristic: &Characteristic) -> Result<(), BleError> {
        let server_service = task::block_on(async {
            self.ble_server.get_service(service_id.to_uuid()).await
        });
        if let Some(service) = server_service {    
            match NimbleProperties::from_bits(characteristic.properties.to_le()) {
                Some(properties) => {
                    let mut charac = service.lock().create_characteristic(
                        characteristic.id.to_uuid(),
                        properties,
                    );
                    charac.lock().set_value(&characteristic.data);
                    return Ok(());
                },
                None => {return Err(BleError::PropertiesError)},
            }
        }
        Err(BleError::ServiceNotFound)
    }

    pub fn start(&mut self) -> Result<(), BleError>{
        self.create_advertisement_data()?;
        self.advertisement.lock().start().map_err(|_| BleError::StartingAdvertisementError)
    }

    fn create_advertisement_data(&mut self) -> Result<(), BleError>{
        let mut adv_data = BLEAdvertisementData::new();
        adv_data.name(&self.advertising_name);
        for service in &self.services {
            adv_data.add_service_uuid(service.id.to_uuid());
        }
        self.advertisement.lock().set_data(&mut adv_data).map_err(|_| BleError::AdvertisementError)
    }
}

impl<'a> InterruptDriver for BleServer<'a>{
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError> {
        let mut all_info = Vec::new();
        while let Some(info) = self.user_on_connection.con_info.lock().unwrap().pop(){
            all_info.push(info);
        }

        let mut callback = self.user_on_connection.callback.take();
        if let Some(ref mut call) = callback{
            for info in all_info{
                call(self, info);
            }
            self.user_on_connection.callback = callback;
        }
        todo!()
    }
}