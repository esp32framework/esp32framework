use esp_idf_svc::sys::configTICK_RATE_HZ;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub type SharableRef<T> = Rc<RefCell<T>>;

/// Trait to handle sherable references.
pub trait SharableRefExt<T> {
    /// Creates a new SharableRef.
    ///
    /// # Arguments
    ///
    /// - `inner`: The value to wrap in a sharable reference.
    ///
    /// # Returns
    ///
    /// A new `SharableRef<T>` wrapping the inner value.
    fn new_sharable(inner: T) -> SharableRef<T>;

    /// Returns a shared reference to the inner value.
    ///
    /// # Returns
    ///
    /// A `Ref<T>` sharing ownership of the inner value.
    fn deref(&self) -> Ref<T>;

    /// Returns a mutable shared reference to the inner value.
    ///
    /// # Returns
    ///
    /// A `RefMut<T>` sharing mutable ownership of the inner value.
    fn deref_mut(&mut self) -> RefMut<T>;
}

impl<T> SharableRefExt<T> for SharableRef<T> {
    fn new_sharable(inner: T) -> SharableRef<T> {
        Rc::new(RefCell::new(inner))
    }
    fn deref_mut(&mut self) -> RefMut<T> {
        self.borrow_mut()
    }
    fn deref(&self) -> Ref<T> {
        self.borrow()
    }
}

/// Converts microseconds to system ticks based on the configured tick rate.
///
/// # Arguments
///
/// * `time_us` - The duration in microseconds.
///
/// # Returns
///
/// The converted duration in ticks using a u32 value.
pub fn micro_to_ticks(time_us: u32) -> u32 {
    ((configTICK_RATE_HZ as u64) * (time_us as u64) / 1_000_000_u64) as u32
}
