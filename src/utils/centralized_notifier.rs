use std::{sync::mpsc::{self, Receiver, Sender}, time::Duration};

pub struct CentralizedNotification{
    sx: Sender<bool>,
    rx: Receiver<bool>,
}

pub struct CentralizedNotifier{
    sx: Sender<bool>
}

#[derive(Debug)]
pub enum CentralizedNotificationError{
    NoOneToReceiveNotification
}

impl CentralizedNotification{
    pub fn new()-> Self{
        let (sx, rx) = mpsc::channel();
        CentralizedNotification{sx, rx}
    }

    pub fn wait_for_notification(&self, duration: Option<Duration>)->Option<bool>{
        match duration {
            Some(dur) => self.rx.recv_timeout(dur).ok(),
            None => Some(self.rx.recv().unwrap()),
        }
    }

    pub fn create_notifier(&self)-> CentralizedNotifier{
        CentralizedNotifier{sx: self.sx.clone()}
    }
}

impl CentralizedNotifier{
    pub fn notify(&self)->Result<(), CentralizedNotificationError>{
        self.sx.send(true).map_err(|_| CentralizedNotificationError::NoOneToReceiveNotification)
    }
}
