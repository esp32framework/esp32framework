use std::num::NonZeroU32;
use std::{collections::HashMap, sync::Arc};
use std::sync::Mutex as Mutex2;

use esp32_nimble::BLEAddress;
use esp32_nimble::{utilities::mutex::Mutex, uuid128, BLEAdvertisementData, BLEAdvertising, BLEConnDesc, BLEDevice, BLEError, BLEServer, NimbleProperties};
use esp_idf_svc::hal::task;
use esp_idf_svc::hal::task::queue::Queue;
use esp_idf_svc::hal::task::notification::Notifier;


use crate::utils::auxiliary::ISRQueue;
use crate::utils::esp32_framework_error::Esp32FrameworkError;
use crate::InterruptDriver;

use super::{BleError, BleId, Characteristic, Service};

pub struct BleServer<'a> {
    advertising_name: String,
    ble_server: &'a mut BLEServer,
    services: Vec<Service>,  // TODO: Change it to Vec<&Service>
    advertisement: &'a Mutex<BLEAdvertising>,
    user_on_connection: Option<ConnectionCallback<'a>>
}

struct ConnectionCallback<'a>{
    callback: Box<dyn FnMut(&mut BleServer<'a>, &ConnectionInformation) + 'a>,
    info_queue: ISRQueue<ConnectionInformation>,
    notifier: Arc<Notifier>
}

impl<'a> ConnectionCallback<'a>{
    fn new(notifier: Arc<Notifier>) -> Self{
        Self { callback: Box::new(|_,_| {}), info_queue: ISRQueue::new(50), notifier}
    }

    fn set_callback<C: FnMut(&mut BleServer<'a>, &ConnectionInformation) + 'a>(&mut self, callback: C){
        self.callback = Box::new(callback);
    }

    fn handle_connection_changes(&mut self, server: &mut BleServer<'a>){
        while let Ok(item)  = self.info_queue.try_recv() {
            (self.callback)(server, &item)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectionInformation{
    address: BLEAddress,
    id_address: BLEAddress,
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
    fn from_BLEConnDesc(desc: &BLEConnDesc, is_connected: bool) -> Self{
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
            user_on_connection: Some(ConnectionCallback::new(notifier)),
            
        };
            
        for service in  &services {
            server.set_service(service);
        }

        server
    }

    pub fn connection_handler<C: FnMut(&mut Self, &ConnectionInformation) + 'a>(&mut self, handler: C) -> &mut Self
    {
        let user_on_connection = self.user_on_connection.as_mut().unwrap();
        let notifier_ref = user_on_connection.notifier.clone();
        let mut con_info_ref = user_on_connection.info_queue.clone();
        user_on_connection.set_callback(handler);
        
        self.ble_server.on_connect(move |_, info| {
            unsafe{ notifier_ref.notify_and_yield(NonZeroU32::new(1).unwrap()); }
            _ = con_info_ref.send_timeout(ConnectionInformation::from_BLEConnDesc(info, true), 1_000_000); //
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

        //self.ble_server.on_disconnect(closure);
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

    /// Set a characteristic to the server. Returns an error if the service does not exist or the properties are invalid.
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
        let mut user_on_connection = self.user_on_connection.take().unwrap();
        user_on_connection.handle_connection_changes(self);
        self.user_on_connection = Some(user_on_connection);
        Ok(())
    }
}