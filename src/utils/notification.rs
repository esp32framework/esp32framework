use std::{future::Future, num::NonZeroU32, ops::Not, rc::Rc, sync::Arc, task::{Context, Poll}, time::{Duration, Instant}};

use esp_idf_svc::hal::task::{asynch::Notification as AsyncNotif, block_on};

use super::{auxiliary::{SharableRef, SharableRefExt}, timer_driver::TimerDriver};

#[derive(Debug)]
pub enum NotificationError{
    NoOneToNotify
}

pub struct Notification{
    notif: Arc<AsyncNotif>,
}

#[derive(Clone)]
pub struct Notifier{
    notif: Arc<AsyncNotif>,
}

impl Notification{
    pub fn new()-> Self{
        Self { notif: Arc::new(AsyncNotif::new()) }
    }

    pub async fn wait(&self){
        self.notif.wait().await;
    }

    pub fn blocking_wait(&self){
        block_on(self.notif.wait());
    }

    pub fn notifier(&self)->Notifier{
        Notifier::from(self)
    }
}

impl From<&Notification> for Notifier{
    fn from(value: &Notification) -> Self {
        Self { notif: value.notif.clone()}
    }
}

impl Notifier{
    pub fn notify(&self)-> Result<(), NotificationError>{
        if self.notif.notify_lsb(){
            Ok(())
        }else {
            Err(NotificationError::NoOneToNotify)
        }
    }
}