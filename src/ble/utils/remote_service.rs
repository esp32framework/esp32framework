use esp32_nimble::{BLERemoteCharacteristic, BLERemoteDescriptor};
use esp_idf_svc::hal::task::block_on;

use crate::utils::{
    auxiliary::{SharableRef, SharableRefExt},
    isr_queues::{ISRByteArrayQueue, ISRQueueTrait},
    notification::Notifier,
};

use super::{BleError, BleId};

/// A remote characteristic representing an available characteristic of a given service of a
/// ble connection. Can be used to read, write and notify.
pub struct RemoteCharacteristic {
    inner: SharableRef<_RemoteCharacteristic>,
    updater: SharableRef<RemoteCharacteristicUpdater>,
}

/// Auxiliary struct used for the updating of characteristics. Most notably execution use callbacks
/// on ble notifies.
#[derive(Default)]
struct RemoteCharacteristicUpdater {
    notify_callback: Option<Box<dyn FnMut(Vec<u8>)>>,
    notify_queue: Option<ISRByteArrayQueue>,
}

/// A remote characteristic representing an available characteristic of a given service of a
/// ble connection. Can be used to read, write and notify.
struct _RemoteCharacteristic {
    characteristic: BLERemoteCharacteristic,
    notifier: Option<Notifier>,
}

impl RemoteCharacteristicUpdater {
    /// If a ble notify was triggered by a ble server of this characteristic, then executes the
    /// user callback if it has been set
    pub fn execute_if_notified(&mut self) {
        if let Some(queue) = self.notify_queue.as_mut() {
            while let Ok(byte_array) = queue.try_recv() {
                if let Some(callback) = self.notify_callback.as_mut() {
                    callback(byte_array)
                }
            }
        }
    }

    /// Returns a clone of the [ISRByteArrayQueue], allowing for communications.
    fn get_queue(&mut self) -> ISRByteArrayQueue {
        match self.notify_queue.as_ref() {
            Some(queue) => queue.clone(),
            None => {
                self.notify_queue = Some(ISRByteArrayQueue::new(50));
                self.notify_queue.as_ref().unwrap().clone()
            }
        }
    }
}

impl RemoteCharacteristic {
    /// Creates a new [RemoteCharacteristic] which is a wrapper of [&mut BLERemoteCharacteristic]
    ///
    /// # Arguments
    ///
    /// - `characteristic`: the remote characteristic to be wrapped
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller]
    ///
    /// # Returns
    /// A [RemoteCharacteristic]
    pub fn new(characteristic: &mut BLERemoteCharacteristic, notifier: Notifier) -> Self {
        Self {
            inner: SharableRef::new_sharable(_RemoteCharacteristic::new(characteristic, notifier)),
            updater: SharableRef::new_sharable(RemoteCharacteristicUpdater::default()),
        }
    }

    /// Sets the user callaback to be triggered when the characteristic gets notified
    ///
    /// # Arguments
    ///
    /// - `callback`: A user callback to be executed when a notification is received. This callback will
    ///   be called with an `Vec<u8>` of the data that was received in the notification
    ///
    /// # Returns
    ///
    /// A `Result` containing () if the operation was completed succesfully, or a [BleError] if
    /// the operation failed
    ///
    /// # Errors
    ///
    /// - `BleError::CharacteristicNotNotifiable`, if attempting to set a callback to a
    ///   non notifiable characteristic
    pub fn on_notify<C: FnMut(Vec<u8>) + 'static>(&mut self, callback: C) -> Result<(), BleError> {
        let queue = self.updater.borrow_mut().get_queue();
        self.inner.borrow_mut().set_notification_on_notify(queue)?;
        self.updater.borrow_mut().notify_callback = Some(Box::new(callback));
        Ok(())
    }

    /// Efectibly clones the remote characteristic, but is only allowed in the crate
    pub(crate) fn duplicate(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            updater: self.updater.clone(),
        }
    }

    /// if a user callback has been set and a notification has been received, then the user
    /// callback will be executed
    pub fn execute_if_notified(&mut self) {
        self.updater.borrow_mut().execute_if_notified()
    }
}

#[sharable_reference_macro::sharable_reference_wrapper]
impl _RemoteCharacteristic {
    /// Creates a new [_RemoteCharacteristic] which is a wrapper of [&mut BLERemoteCharacteristic]
    ///
    /// # Arguments
    ///
    /// - `characteristic`: the remote characteristic to be wrapped
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller]
    ///
    /// # Returns
    /// A [_RemoteCharacteristic]
    fn new(characteristic: &mut BLERemoteCharacteristic, notifier: Notifier) -> Self {
        Self {
            characteristic: characteristic.clone(),
            notifier: Some(notifier),
        }
    }

    /// Returns the id of the characteristic
    pub fn id(&self) -> BleId {
        BleId::from(self.characteristic.uuid())
    }

    /// Returns wheter the characteristic is readable or not
    pub fn is_readable(&self) -> bool {
        self.characteristic.can_read()
    }

    /// Returns wheter the characteristic is writable or not
    pub fn is_writable(&self) -> bool {
        self.characteristic.can_write()
    }

    /// Returns wheter the characteristic is notifiable or not
    pub fn is_notifiable(&self) -> bool {
        self.characteristic.can_notify()
    }

    /// Returns wheter the characteristic is broadcastable or not
    pub fn is_broadcastable(&self) -> bool {
        self.characteristic.can_broadcast()
    }

    /// Returns wheter the characteristic is writable with no response or not
    pub fn is_writable_no_resp(&self) -> bool {
        self.characteristic.can_write_no_response()
    }

    /// Non blocking async version of [Self::read]
    pub async fn read_async(&mut self) -> Result<Vec<u8>, BleError> {
        if !self.is_readable() {
            return Err(BleError::CharacteristicNotReadable);
        }
        self.characteristic
            .read_value()
            .await
            .map_err(BleError::from_characteristic_context)
    }

    /// Attempts to read the characteristics value
    ///
    /// # Returns     
    ///
    /// A `Result` contanting the bytes read as a `Vec<u8>`, or a `BleError`
    ///
    /// # Errors
    ///
    /// `BleError::CharacteristicNotReadable`: If the characteristic is not readable
    /// `BleError::Disconnected`: If connection to the ble server is lost
    /// `BleError::Code`: On other errors
    pub fn read(&mut self) -> Result<Vec<u8>, BleError> {
        block_on(self.read_async())
    }

    /// Non blocking async version of [Self::write]
    pub async fn write_async(&mut self, data: &[u8]) -> Result<(), BleError> {
        if !self.is_writable() && !self.is_writable_no_resp() {
            return Err(BleError::CharacteristicNotWritable);
        }
        self.characteristic
            .write_value(data, !self.is_writable_no_resp())
            .await
            .map_err(BleError::from_characteristic_context)
    }

    /// Attempts to write the characteristic value. The write will wait for a response or not depending if the characterisitc
    /// [Self::is_writable_no_resp], or not.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the operation was successfull or BleError on failure
    ///
    /// # Error
    /// `BleError::CharacteristicNotWritable`: If the characteristic is not writable
    /// `BleError::Disconnected`: If connection to the ble server is lost
    /// `BleError::Code`: On other errors
    pub fn write(&mut self, data: &[u8]) -> Result<(), BleError> {
        block_on(self.write_async(data))
    }

    /// Documented on [RemoteCharacteristic::on_notify]
    fn set_notification_on_notify(&mut self, mut queue: ISRByteArrayQueue) -> Result<(), BleError> {
        if !self.is_notifiable() {
            return Err(BleError::CharacteristicNotNotifiable);
        }
        if let Some(notifier) = self.notifier.take() {
            self.characteristic.on_notify(move |bytes| {
                notifier.notify();
                queue.send(bytes.to_vec())
            });
        }
        Ok(())
    }

    /// Attempts to get the specified descriptor of the characteristic
    ///
    /// # Arguments
    ///
    /// `id`: id of the descripor to look for
    ///
    /// # Returns
    ///
    /// A `Result` with the `RemoteDescriptor` on success and a `BleError` on failure
    ///
    /// # Errors
    ///
    /// `BleError::DescriptorNotFound`: if characteristic does not contain the desired descriptor
    /// `BleError::Disconnected`: If connection to the ble server is lost
    /// `BleError::Code`: On other errors
    pub fn get_descriptor(&mut self, id: &BleId) -> Result<RemoteDescriptor, BleError> {
        block_on(self.get_descriptor_async(id))
    }

    /// Non blocking async version of [Self::get_descriptor]
    pub async fn get_descriptor_async(&mut self, id: &BleId) -> Result<RemoteDescriptor, BleError> {
        let remote_descriptor = self
            .characteristic
            .get_descriptor(id.to_uuid())
            .await
            .map_err(BleError::from_descriptors_context)?;
        Ok(RemoteDescriptor::from(remote_descriptor))
    }

    /// Attempts to get the all descriptors of the characteristic
    ///
    /// # Returns
    ///
    /// A `Result` with  `Vec<RemoteDescriptor>` on success and a `BleError` on failure
    ///
    /// # Errors
    ///
    /// `BleError::Disconnected`: If connection to the ble server is lost
    /// `BleError::Code`: On other errors
    pub fn get_all_descriptors(&mut self) -> Result<Vec<RemoteDescriptor>, BleError> {
        block_on(self.get_all_descriptors_async())
    }

    /// Non blocking async version of [Self::get_all_descriptors]
    pub async fn get_all_descriptors_async(&mut self) -> Result<Vec<RemoteDescriptor>, BleError> {
        let remote_descriptors = self.characteristic.get_descriptors().await?;
        Ok(remote_descriptors.map(RemoteDescriptor::from).collect())
    }
}

/// A remote descriptor representing an available descriptor of a given characteristic.
/// Can be used to read and write.
pub struct RemoteDescriptor {
    descriptor: BLERemoteDescriptor,
}

impl RemoteDescriptor {
    /// Returns the id of the descriptor
    pub fn id(&self) -> BleId {
        BleId::from(self.descriptor.uuid())
    }

    /// Non blocking async version of [Self::read]
    pub async fn read_async(&mut self) -> Result<Vec<u8>, BleError> {
        self.descriptor
            .read_value()
            .await
            .map_err(BleError::from_descriptors_context)
    }

    /// Attempts to read the descriptor's value
    ///
    /// # Returns     
    ///
    /// A `Result` contanting the bytes read as a `Vec<u8>`, or a `BleError`
    ///
    /// # Errors
    ///
    /// `BleError::DescriptorNotReadable`: If the descriptor is not readable
    /// `BleError::Disconnected`: If connection to the ble server is lost
    /// `BleError::Code`: On other errors
    pub fn read(&mut self) -> Result<Vec<u8>, BleError> {
        block_on(self.read_async())
    }

    /// Non blocking async version of [Self::write]
    pub async fn write_async(&mut self, data: &[u8]) -> Result<(), BleError> {
        self.descriptor
            .write_value(data, true)
            .await
            .map_err(BleError::from_descriptors_context)
    }

    /// Attempts to write data to the descriptor
    ///
    /// # Arguments
    ///
    /// `data`: data to be written
    ///
    /// # Returns     
    ///
    /// A `Result` contanting `()` if successfull, or a `BleError` on failure
    ///
    /// # Errors
    ///
    /// `BleError:BleError::DescriptorNotWritable:`: If the descriptor is not readable
    /// `BleError::Disconnected`: If connection to the ble server is lost
    /// `BleError::Code`: On other errors
    pub fn write(&mut self, data: &[u8]) -> Result<(), BleError> {
        block_on(self.write_async(data))
    }
}

impl From<&mut BLERemoteDescriptor> for RemoteDescriptor {
    fn from(value: &mut BLERemoteDescriptor) -> Self {
        Self {
            descriptor: value.clone(),
        }
    }
}
