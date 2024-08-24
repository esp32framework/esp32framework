use std::{cell::{Ref, RefCell, RefMut}, ops::Deref, sync::Arc, rc::Rc};
pub type SharableRef<T> = Rc<RefCell<T>>;
use esp_idf_svc::{hal::{delay::BLOCK, task::queue::Queue}, sys::{EspError, configTICK_RATE_HZ}};
use esp_idf_svc::sys::TickType_t;

#[derive(Debug)]
pub enum ISRQueueError {
    Timeout,
    Empty,
}

pub trait SharableRefExt<T>{
    fn new_sharable(inner: T) -> SharableRef<T>;
    
    fn deref(&self) -> Ref<T>;
        
    fn deref_mut(&mut self) -> RefMut<T>;
}

impl<T> SharableRefExt<T> for SharableRef<T>{
    fn new_sharable(inner: T) -> SharableRef<T>{
        Rc::new(RefCell::new(inner))
    }
    fn deref_mut(&mut self) -> RefMut<T> {
        self.borrow_mut()
    }
    fn deref(&self) -> Ref<T>{
        self.borrow()
    }
}

#[derive(Clone)]
pub struct ISRQueue<T: Copy>{
    q: Arc<Queue<T>>
}

impl<T: Copy> ISRQueue<T>{
    pub fn new(size: usize)-> Self{
        Self{q: Arc::new(Queue::new(size))}
    }

    pub fn send(&mut self, item: T){
        self.send_timeout(item, BLOCK).unwrap()
    }
    
    pub fn try_send(&mut self, item: T) -> Result<(), ISRQueueError>{
        self.send_timeout(item, 0).map_err(|_| ISRQueueError::Empty)
    }

    pub fn send_timeout(&mut self,item: T, micro: u32)-> Result<(), ISRQueueError> {
        match self.q.send_back(item, micro_to_ticks(micro)) {
            Ok(_) => Ok(()), 
            Err(_) => Err(ISRQueueError::Timeout)
        }
    }

    pub fn receive(&mut self) -> T {
        self.receive_timeout(BLOCK).unwrap()
    }

    pub fn try_recv(&mut self) -> Result<T, ISRQueueError> {
        self.receive_timeout(0).map_err(|_| ISRQueueError::Empty)
    }

    pub fn receive_timeout(&mut self, micro: u32)-> Result<T, ISRQueueError> {
        match self.q.recv_front(micro_to_ticks(micro)){
            Some((item, _)) => Ok(item), 
            None => Err(ISRQueueError::Timeout)
        }
    }
}

/// Converts microseconds to system ticks based on the configured tick rate.
pub fn micro_to_ticks(time_us: u32) -> u32 {
    ((configTICK_RATE_HZ as u64) * (time_us as u64) / (1_000_000 as u64)) as u32
}