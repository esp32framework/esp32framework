use esp_idf_svc::hal::task::{asynch::Notification as AsyncNotif, block_on};
use std::sync::Arc;

/// Used for receiving a notification from an ISR context
pub struct Notification {
    notif: Arc<AsyncNotif>,
}

#[derive(Clone)]
/// Used for sending a notification from an ISR context
pub struct Notifier {
    notif: Arc<AsyncNotif>,
}

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}

impl Notification {
    pub fn new() -> Self {
        Self {
            notif: Arc::new(AsyncNotif::new()),
        }
    }

    /// Async version of [Self::blocking_wait]
    pub async fn wait(&self) {
        self.notif.wait().await;
    }

    /// Polls for a notification
    ///
    /// # Returns
    ///
    /// false: if there is no notification
    /// true: if there is a notification, this also consumes the notification
    pub fn poll(&self) -> bool {
        block_on(self._poll())
    }

    async fn _poll(&self) -> bool {
        futures::poll!(self.notif.wait()).is_ready()
    }

    /// Blocking waits for a notification sent by any of the notification's notifiers
    pub fn blocking_wait(&self) {
        block_on(self.notif.wait());
    }

    /// Create a notifier for this notification.
    ///
    /// # Returns
    ///
    /// A `Notifier` capable of sending notifications to this `Notification`. There can be multiple
    /// `Notifiers` for a single `Notification`.
    pub fn notifier(&self) -> Notifier {
        Notifier::from(self)
    }
}

impl From<&Notification> for Notifier {
    fn from(value: &Notification) -> Self {
        Self {
            notif: value.notif.clone(),
        }
    }
}

impl Notifier {
    /// Send a notification to the associated `Notification`, this will wake the notification if it is
    /// currently blocked in a wait
    pub fn notify(&self) -> bool {
        self.notif.notify_lsb()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_notif_01_notify() {
        let notif = Notification::new();
        notif.notifier().notify();
        assert!(notif.poll())
    }

    #[test]
    fn test_notif_02_poll_consumes_notification() {
        let notif = Notification::new();
        notif.notifier().notify();
        assert!(notif.poll());
        assert!(!notif.poll());
    }
}
