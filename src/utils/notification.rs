use std::sync::Arc;

use esp_idf_svc::hal::task::{asynch::Notification as AsyncNotif, block_on};


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

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}

impl Notification{
    pub fn new()-> Self{
        Self { notif: Arc::new(AsyncNotif::new()) }
    }

    pub async fn wait(&self){
        self.notif.wait().await;
        println!("Recibi notif");
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
    pub fn notify(&self)-> bool{
        self.notif.notify_lsb()
    }
}