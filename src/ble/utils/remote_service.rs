use esp32_nimble::{BLERemoteCharacteristic, BLERemoteDescriptor};
use esp_idf_svc::hal::task::block_on;

use crate::utils::{auxiliary::{ISRByteArrayQueue, ISRQueueTrait, SharableRef, SharableRefExt}, notification::Notifier};

use super::{BleError, BleId};

pub struct RemoteCharacteristic{
    inner: SharableRef<_RemoteCharacteristic>,
    updater: SharableRef<RemoteCharacteristicUpdater>
}

#[derive(Default)]
struct RemoteCharacteristicUpdater{
    notify_callback: Option<Box<dyn FnMut(Vec<u8>)>>,
    notify_queue: Option<ISRByteArrayQueue>,
}

struct _RemoteCharacteristic{
    characteristic: BLERemoteCharacteristic,
    notifier: Option<Notifier>,
}

impl RemoteCharacteristicUpdater{
    pub fn update_interrupt(&mut self){
        if let Some(queue) = self.notify_queue.as_mut(){
            while let Ok(byte_array) = queue.try_recv(){
                if let Some(callback) = self.notify_callback.as_mut(){
                    callback(byte_array)
                }
            }
        }
    }

    fn get_queue(&mut self)-> ISRByteArrayQueue{
        match self.notify_queue.as_ref(){
            Some(queue) => queue.clone(),
            None => {
                self.notify_queue = Some(ISRByteArrayQueue::new(50));
                self.notify_queue.as_ref().unwrap().clone()
            },
        }
    }
}

impl RemoteCharacteristic{
    pub fn new(characteristic: &mut BLERemoteCharacteristic, notifier: Notifier)-> Self{
        Self {
            inner: SharableRef::new_sharable(_RemoteCharacteristic::new(characteristic, notifier)),
            updater: SharableRef::new_sharable(RemoteCharacteristicUpdater::default())
        }
    }

    pub fn on_notify<C: FnMut(Vec<u8>) + 'static>(&mut self, callback: C)->Result<(), BleError>{
        let queue = self.updater.borrow_mut().get_queue();
        self.inner.borrow_mut().set_notification_on_notify(queue)?;
        self.updater.borrow_mut().notify_callback = Some(Box::new(callback));
        Ok(())
    }

    pub(crate) fn duplicate(&self)->Self{
        Self { inner: self.inner.clone(), updater: self.updater.clone() }
    }

    pub fn update_interrupt(&mut self){
        self.updater.borrow_mut().update_interrupt()
    }
}

#[sharable_reference_macro::sharable_reference_wrapper]
impl _RemoteCharacteristic{
    fn new(characteristic: &mut BLERemoteCharacteristic, notifier: Notifier)-> Self{
        Self { characteristic: characteristic.clone(), notifier: Some(notifier)}
    }

    pub fn id(&self)-> BleId{
        BleId::from(self.characteristic.uuid())
    }

    pub fn is_readable(&self)-> bool{
        self.characteristic.can_read()
    }

    pub fn is_writable(&self)->bool{
        self.characteristic.can_write()
    }

    pub fn is_notifiable(&self)->bool{
        self.characteristic.can_notify()
    }

    pub fn is_broadcastable(&self)->bool{
        self.characteristic.can_broadcast()
    }
    
    pub fn is_writable_no_resp(&self)->bool{
        self.characteristic.can_write_no_response()
    }

    pub async fn read_async(&mut self) -> Result<Vec<u8> ,BleError>{
        if !self.is_readable(){
            return Err(BleError::CharacteristicIsNotReadable)
        }
        self.characteristic.read_value().await.map_err(BleError::from)
    }
    
    pub fn read(&mut self) -> Result<Vec<u8> ,BleError>{
        block_on(self.read_async())
    }

    pub async fn write_async(&mut self, data: &[u8]) -> Result<() ,BleError>{
        if !self.is_writable() && !self.is_writable_no_resp(){
            return Err(BleError::CharacteristicIsNotWritable)
        }
        self.characteristic.write_value(data, !self.is_writable_no_resp()).await.map_err(BleError::from)
    }
    
    pub fn write(&mut self, data: &[u8]) -> Result<(), BleError> {
        block_on(self.write_async(data))
    }
    
    fn set_notification_on_notify(&mut self, mut queue: ISRByteArrayQueue)->Result<(), BleError>{
        if !self.is_notifiable(){
            return Err(BleError::CharacteristicIsNotNotifiable)
        }
        if let Some(notifier) = self.notifier.take(){
            self.characteristic.on_notify(move |bytes| {
                notifier.notify();
                queue.send(bytes.to_vec())
            });
        }
        Ok(())
    }
    
    pub fn get_descriptor(&mut self, id: &BleId) -> Result<RemoteDescriptor, BleError>{
        block_on(self.get_descriptor_async(id))
    }

    pub async fn get_descriptor_async(&mut self, id: &BleId) -> Result<RemoteDescriptor, BleError>{
        let remote_descriptor = self.characteristic.get_descriptor(id.to_uuid()).await?;
        Ok(RemoteDescriptor::from(remote_descriptor))
    }

    pub fn get_all_descriptors(&mut self) -> Result<Vec<RemoteDescriptor>, BleError>{
        block_on(self.get_all_descriptors_async())
    }

    pub async fn get_all_descriptors_async(&mut self) -> Result<Vec<RemoteDescriptor>, BleError>{
        let remote_descriptors = self.characteristic.get_descriptors().await?;
        Ok(remote_descriptors.map(RemoteDescriptor::from).collect())
    }
}

pub struct RemoteDescriptor{
    descriptor: BLERemoteDescriptor
}

impl RemoteDescriptor{
    pub fn id(&self)-> BleId{
        BleId::from(self.descriptor.uuid())
    }

    pub async fn read_async(&mut self)-> Result<Vec<u8>, BleError>{
        self.descriptor.read_value().await.map_err(BleError::from)
    }
    
    pub fn read(&mut self)-> Result<Vec<u8>, BleError>{
        block_on(self.read_async())
    }

    pub async fn write_async(&mut self, data: &[u8]) -> Result<(), BleError> {
        self.descriptor.write_value(data, false).await.map_err(BleError::from)
    }

    pub fn write(&mut self, data: &[u8]) -> Result<(), BleError> {
        block_on(self.write_async(data))
    }
}

impl From<&mut BLERemoteDescriptor> for RemoteDescriptor{
    fn from(value: &mut BLERemoteDescriptor) -> Self {
        Self { descriptor: value.clone() }
    }
}