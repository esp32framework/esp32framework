use crate::{
    microcontroller_src::{
        interrupt_driver::InterruptDriver,
        peripherals::{Peripheral, PeripheralError},
    },
    utils::timer_driver::timer::TimerConfig,
};
use esp_idf_svc::hal::timer;
use sharable_reference_macro::sharable_reference_wrapper;
use std::{
    collections::{BinaryHeap, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use super::{
    auxiliary::{SharableRef, SharableRefExt},
    esp32_framework_error::Esp32FrameworkError,
    notification::{Notification, Notifier},
};

const MICRO_IN_SEC: u64 = 1000000;
const MAX_CHILDREN: u16 = u8::MAX as u16;

/// Driver for handling the underlying timer resource. There can be multiple [TimerDriver]s with the same underlying
/// timer resource, but they will function as if each one had a diferent timer resource.
pub struct TimerDriver<'a> {
    inner: SharableRef<_TimerDriver<'a>>,
    id: u16,
    next_child: u16,
}

/// Driver for handling the timer resource, allowing for multiple interrupts to be set, deppending on the id received.
/// Each reference has a unique id and can create one interrupt each. This is the inner of [TimerDriver] which toghether
/// give the ilution of multiple timer resources when in reality there is only one.
struct _TimerDriver<'a> {
    driver: timer::TimerDriver<'a>,
    interrupt_update: InterruptUpdate,
    alarms: BinaryHeap<Alarm>,
    interrupts: HashMap<u16, TimeInterrupt>,
}


#[derive(Debug, PartialEq)]
pub enum TimerDriverError {
    CannotSetTimerCounter,
    CouldNotSetTimer,
    ErrorReadingAlarm,
    ErrorReadingTimer,
    ErrorSettingUpForDelay,
    InvalidPeripheral(PeripheralError),
    OnlyOriginalCopyCanCreateChildren,
    SubscriptionError,
    TooManyChildren,
}

/// Represents an interrupt to be executed after some time a number of times
struct TimeInterrupt {
    after: u64,
    id: u16,
    current_alarm_id: usize,
    status: TimerInterruptStatus,
    remaining_triggers: Option<u32>,
    auto_reenable: bool,
    callback: Box<dyn FnMut()>,
}

#[derive(Debug, PartialEq, Eq)]
enum TimerInterruptStatus {
    Disabled,
    Enabled,
    Removing,
}

#[derive(Debug, PartialEq, Eq)]
struct Alarm {
    time: u64,
    id: u16,
    alarm_id: usize,
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
#[derive(Debug, Clone)]
struct InterruptUpdate {
    update: Arc<AtomicBool>,
}

impl InterruptUpdate {
    fn new() -> InterruptUpdate {
        InterruptUpdate {
            update: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Checks for an update
    fn any_updates(&self) -> bool {
        self.update.load(Ordering::Relaxed)
    }

    /// Sets an update on the interrupt update
    fn new_update(&self) {
        self.update.store(true, Ordering::Relaxed);
    }

    /// Removes update
    fn handling_update(&self) {
        self.update.store(false, Ordering::Relaxed);
    }

    /// If there are any updates it handles them
    fn handle_any_updates(&self) -> bool {
        if self.any_updates() {
            self.handling_update();
            true
        } else {
            false
        }
    }
}

impl TimeInterrupt {
    fn new(
        id: u16,
        callback: Box<dyn FnMut()>,
        time: u64,
        amount_of_triggers: Option<u32>,
        auto_reenable: bool,
    ) -> TimeInterrupt {
        TimeInterrupt {
            after: time,
            id,
            current_alarm_id: 0,
            status: TimerInterruptStatus::Disabled,
            remaining_triggers: amount_of_triggers,
            auto_reenable,
            callback,
        }
    }

    /// Creates the corresponding alarm
    fn get_alarm(&self, current_time: u64) -> Alarm {
        Alarm::new(self.id, self.current_alarm_id, self.after + current_time)
    }

    /// Makes it so all previouse alarms are ignored, by advancing the alarm id
    fn disable_previouse_alarms(&mut self) {
        self.current_alarm_id += 1
    }

    /// If any triggers remain execute the callback
    fn trigger(&mut self) {
        if let Some(ref mut amount) = self.remaining_triggers {
            if *amount == 0 {
                return;
            }
            *amount -= 1;
        }
        (self.callback)();
        self.status = TimerInterruptStatus::Disabled;
    }

    /// Checks if there are any triggers left or there was no limit set to the amount of triggers
    fn any_triggers_left(&self) -> bool {
        match self.remaining_triggers {
            Some(triggers) => triggers > 0,
            None => true,
        }
    }
}

impl Alarm {
    fn new(id: u16, alarm_id: usize, time: u64) -> Self {
        Alarm { time, id, alarm_id }
    }
}

impl Ord for Alarm {
    // Order is inverted for insertion as minimal heap
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.time.cmp(&other.time) {
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

impl PartialOrd for Alarm {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[sharable_reference_wrapper("id")]
impl<'a> _TimerDriver<'a> {
    /// Create a new `_TimerDriver` to handle one of the underlying timer groups
    ///
    /// # Arguments
    ///
    /// - `timer`: A timer Peripheral.
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller] after an interrupt
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_TimerDriver` instance, or a `TimerDriverError` if initialization fails.
    ///
    /// # Errors
    ///
    /// - `TimerDrivererror::InvalidPeripheral`: If per parameter is not a timer or, the timer has been already taken
    /// - `TimerDriverError::SubscriptionError`: If it failed while subscribing the base callback
    fn new(timer: Peripheral, notifier: Notifier) -> Result<_TimerDriver<'a>, TimerDriverError> {
        let driver = match timer {
            Peripheral::Timer(timer_num) => match timer_num {
                0 => timer::TimerDriver::new(unsafe { timer::TIMER00::new() }, &TimerConfig::new()),
                1 => timer::TimerDriver::new(unsafe { timer::TIMER10::new() }, &TimerConfig::new()),
                _ => {
                    return Err(TimerDriverError::InvalidPeripheral(
                        PeripheralError::NotATimerGroup,
                    ))
                }
            }
            .map_err(|_| TimerDriverError::InvalidPeripheral(PeripheralError::NotATimerGroup))?,
            Peripheral::None => {
                return Err(TimerDriverError::InvalidPeripheral(
                    PeripheralError::AlreadyTaken,
                ))
            }
            _ => {
                return Err(TimerDriverError::InvalidPeripheral(
                    PeripheralError::NotATimerGroup,
                ))
            }
        };

        let mut timer = _TimerDriver {
            driver,
            interrupt_update: InterruptUpdate::new(),
            alarms: BinaryHeap::new(),
            interrupts: HashMap::new(),
        };
        timer.set_interrupt_update_callback(notifier).map(|_| timer)
    }

    /// Sets the callback for the timer_driver which will modify the interrupt update
    ///
    /// # Arguments
    ///
    /// - `notifier`: A notifier in order to wake up the [crate::Microcontroller] after an interrupt
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `_TimerDriver` instance, or a `TimerDriverError` if initialization fails.
    ///
    /// # Errors
    ///
    /// `TimerDriverError::SubscriptionError`: If it failed while subscribing the base callback
    fn set_interrupt_update_callback(
        &mut self,
        notifier: Notifier,
    ) -> Result<(), TimerDriverError> {
        let interrupt_update_ref = self.interrupt_update.clone();
        let alarm_callback = move || {
            interrupt_update_ref.new_update();
            notifier.notify();
        };
        unsafe {
            self.driver
                .subscribe(alarm_callback)
                .map_err(|_| TimerDriverError::SubscriptionError)
        }
    }

    /// Sets an interrupt that triggers once after "microseconds". For this to start working [Self::enable()]
    /// must be called. After the interrupt has been trigger it can be reset by calling [Self::enable()]
    ///
    /// # Arguments
    ///
    //   - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    ///  - `micro_seconds`: time after which the interrupt will trigger
    ///  - `callback`: callback to be executed when the interrupt triggers
    pub fn interrupt_after<F: FnMut() + 'static>(
        &mut self,
        id: u16,
        micro_seconds: u64,
        callback: F,
    ) {
        self.interrupt_after_n_times(id, micro_seconds, None, false, callback)
    }

    /// Sets an interrupt to trigger every `micro_seconds` for an `amount_of_triggers` if given, if not
    /// triggers indefinitely. If autoenable is set, after triggering the callback, it will be set again
    /// if not it will have to be reenabled manually by caling [Self::enable()]. For this to start working
    /// [Self::enable()] must be called.
    ///
    /// # Arguments
    ///
    //   - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    ///  - `micro_seconds`: time after which the interrupt will trigger
    ///  - `amount_of_triggers`: amount of times the interrupt will trigger, if None it will trigger indefinitely
    ///  - `auto_reenable`: true if the interrupt will be reenabled after triggering
    ///  - `callback`: callback to be executed each time the interrupt triggers
    pub fn interrupt_after_n_times<F: FnMut() + 'static>(
        &mut self,
        id: u16,
        micro_seconds: u64,
        amount_of_triggers: Option<u32>,
        auto_reenable: bool,
        callback: F,
    ) {
        let time = self.micro_to_counter(micro_seconds);
        let mut interrupt = TimeInterrupt::new(
            id,
            Box::new(callback),
            time,
            amount_of_triggers,
            auto_reenable,
        );

        if let Some(old_interrupt) = self.interrupts.get(&id) {
            interrupt.current_alarm_id = old_interrupt.current_alarm_id + 1
        }
        self.interrupts.insert(id, interrupt);
    }

    /// Transforms microseconds to the microcontroller tick_hz
    fn micro_to_counter(&self, micro_seconds: u64) -> u64 {
        (micro_seconds as u128 * self.driver.tick_hz() as u128).min(u64::MAX as u128) as u64
            / MICRO_IN_SEC
    }

    /// Activates the timeInterrupt corresponding to "id". By setting the interrupt status as `TimerInterruptStatus::Enabled`
    /// and making sure the interrupt has an alarm
    ///
    /// # Arguments
    ///   - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    ///
    /// # Returns
    ///
    /// A `Result` with `Ok` if the activation was completed succesfully or an Err(TimerDriverError) if it failed
    ///
    /// # Errors
    ///
    /// - TimerDriverError::ErrorReadingTimer: if it fails when trying to get the current time
    fn activate(&mut self, id: u16) -> Result<(), TimerDriverError> {
        if let Some(interrupt) = self.interrupts.get_mut(&id) {
            if interrupt.status == TimerInterruptStatus::Disabled {
                let current_time = self
                    .driver
                    .counter()
                    .map_err(|_| TimerDriverError::ErrorReadingTimer)?;
                self.alarms.push(interrupt.get_alarm(current_time))
            }
            interrupt.status = TimerInterruptStatus::Enabled
        }
        Ok(())
    }

    /// Deactivates the timeInterrupt corresponding to "id", by setting interrupt status as `TimerInterruptStatus::Disabled`
    /// and making sure all previouse alarms of the interrupt are ignored
    ///
    /// # Arguments
    ///   - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    fn deactivate(&mut self, id: u16) {
        if let Some(interrupt) = self.interrupts.get_mut(&id) {
            if interrupt.status == TimerInterruptStatus::Enabled {
                interrupt.disable_previouse_alarms()
            }
            interrupt.status = TimerInterruptStatus::Disabled
        }
    }

    /// Resets all inner auxiliary structures
    fn reset(&mut self) {
        self.interrupts = HashMap::new();
        self.interrupt_update.handling_update();
        self.alarms = BinaryHeap::new();
    }

    /// Enables or disables the interrupt corresponding to "id". If the interrupt is enabled, and it
    /// is the new lowest time, the soonest alarm is updated. When the first interrupt is enabled, or the last
    /// disabled the timer is started or stoped accordingly
    ///
    /// # Arguments
    /// - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    /// - `enable`: if set true, enable interrupt, if set false, disable it
    ///
    /// # Returns
    /// A `Result` containing `Ok` if interrupt was inabled or `Err(TimerDriverError)` if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::CouldNotSetTimer`: if it fails trying to set an alarm for the interrupt
    /// - `TimerDriverError::ErrorReadingTimer`: if it fails when trying to get the current time
    /// - `TimerDriverError::ErrorReadingAlarm`: failure getting current alarm time
    fn _enable(&mut self, id: u16, enable: bool) -> Result<(), TimerDriverError> {
        let starting_len = self.alarms.len();
        if enable {
            self.activate(id)?;
            self.set_lowest_alarm()?;
        } else {
            self.deactivate(id);
        }

        if self.alarms.is_empty() || starting_len == 0 {
            if enable {
                self.driver
                    .enable_interrupt()
                    .map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            } else {
                self.driver
                    .disable_interrupt()
                    .map_err(|_| TimerDriverError::CouldNotSetTimer)?;
                self.reset()
            }
            self.driver
                .enable_alarm(enable)
                .map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            self.driver
                .enable(enable)
                .map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }
        Ok(())
    }

    /// Enables the interrupt if it has been set.
    ///
    // # Arguments
    // - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    ///
    /// # Returns
    /// A `Result` containing `Ok` if interrupt was inabled or `Err(TimerDriverError)` if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::CouldNotSetTimer`: if it fails trying to set an alarm for the interrupt
    /// - `TimerDriverError::ErrorReadingTimer`: if it fails when trying to get the current time
    /// - `TimerDriverError::ErrorReadingAlarm`: failure getting current alarm time
    pub fn enable(&mut self, id: u16) -> Result<(), TimerDriverError> {
        self._enable(id, true)
    }

    /// Disables the interrupt if it has been set.
    ///
    // # Arguments
    // - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    ///
    /// # Returns
    /// A `Result` containing `Ok` if interrupt was inabled or `Err(TimerDriverError)` if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::CouldNotSetTimer`: if it fails trying to set an alarm for the interrupt
    /// - `TimerDriverError::ErrorReadingTimer`: if it fails when trying to get the current time
    /// - `TimerDriverError::ErrorReadingAlarm`: failure getting current alarm time
    /// Disables the interrupt corresponding to "id". When the last disabled the timer is stopped
    pub fn disable(&mut self, id: u16) -> Result<(), TimerDriverError> {
        self._enable(id, false)
    }

    /// Removes the interrupt if it has been set.
    ///
    // # Arguments
    // - `id`: id by which the interrupt will be identified. This corresponds to the id of the wrapper [TimerDriver]
    ///
    /// # Returns
    /// A `Result` containing `Ok` if interrupt was inabled or `Err(TimerDriverError)` if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::CouldNotSetTimer`: if it fails trying to set an alarm for the interrupt
    /// - `TimerDriverError::ErrorReadingTimer`: if it fails when trying to get the current time
    /// - `TimerDriverError::ErrorReadingAlarm`: failure getting current alarm time
    /// Disables the interrupt corresponding to "id". When the last disabled the timer is stopped
    pub fn remove_interrupt(&mut self, id: u16) -> Result<(), TimerDriverError> {
        self.disable(id)?;
        if let Some(interrupt) = self.interrupts.get_mut(&id) {
            interrupt.status = TimerInterruptStatus::Removing;
        }
        Ok(())
    }

    /// Sets the interrupt to trigger on the soonest alarm
    ///
    /// # Returns
    ///
    /// A `Result` with `Ok` if it was able to set the lowest alarm or `Err(TimerDriverError)` on failure
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::ErrorReadingAlarm`: failure getting current alarm time
    /// - `TimerDriverError::CouldNotSetTimer``: if it fails trying to set an alarm for the interrupt
    fn set_lowest_alarm(&mut self) -> Result<(), TimerDriverError> {
        if let Some(alarm) = self.alarms.peek() {
            if alarm.time
                != self
                    .driver
                    .alarm()
                    .map_err(|_| TimerDriverError::ErrorReadingAlarm)?
            {
                self.driver
                    .set_alarm(alarm.time)
                    .map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            }
            self.driver
                .enable_alarm(true)
                .map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }
        Ok(())
    }

    /// Triggers the callback of a `TimeInterrupt` if there is one enabled interrupt with the same alarm id as `alarm`
    ///
    /// # Arguments
    ///
    /// - alarm: The alarm that triggered and may make an interrupt trigger its callback
    ///
    /// # Returns
    ///
    /// A `Result` with `Ok` if the alarm was handled, triggering the interrupt if conditions are met or an Err(TimerDriverError) if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::ErrorReadingTimer`: if it fails when trying to get the current time
    fn handle_alarm_update(&mut self, alarm: Alarm) -> Result<(), TimerDriverError> {
        if let Some(interrupt) = self.interrupts.get_mut(&alarm.id) {
            if interrupt.current_alarm_id == alarm.alarm_id {
                match interrupt.status {
                    TimerInterruptStatus::Enabled => {
                        interrupt.trigger();
                        if interrupt.any_triggers_left() && interrupt.auto_reenable {
                            self.activate(alarm.id)?;
                        }
                    }
                    TimerInterruptStatus::Disabled => {}
                    TimerInterruptStatus::Removing => {
                        self.interrupts.remove(&alarm.id);
                    }
                }
            }
        }
        Ok(())
    }

    /// Handles the updates of any alarms which have gone off by calling `Self::handle_alarm_update` on any of them, and triggering interrupt callbacks
    /// when needed
    ///
    /// # Returns
    ///
    /// A `Result` with `Ok` if all the alarms were handled correctly or an Err(TimerDriverError) if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::ErrorReadingTimer`: if it fails when trying to get the current time
    /// - `TimerDriverError::CouldNotSetTimer`: if it fails trying to set an alarm for the interrupt
    fn _update_interrupt(&mut self) -> Result<(), TimerDriverError> {
        while self.interrupt_update.handle_any_updates() {
            if let Some(alarm) = self.alarms.pop() {
                self.handle_alarm_update(alarm)?;
            }
            self.set_lowest_alarm()?;
        }
        Ok(())
    }
}

impl<'a> InterruptDriver<'a> for TimerDriver<'a> {
    fn update_interrupt(&mut self) -> Result<(), Esp32FrameworkError> {
        self.inner.deref_mut()._update_interrupt()?;
        Ok(())
    }

    fn get_updater(&self) -> Box<dyn InterruptDriver<'a> + 'a> {
        Box::new(Self {
            inner: self.inner.clone(),
            id: self.id,
            next_child: self.next_child,
        })
    }
}

impl<'a> TimerDriver<'a> {
    pub(crate) fn new(
        timer: Peripheral,
        notifier: Notifier,
    ) -> Result<TimerDriver<'a>, TimerDriverError> {
        Ok(TimerDriver {
            inner: SharableRef::new_sharable(_TimerDriver::new(timer, notifier)?),
            id: 0,
            next_child: 1,
        })
    }

    /// This function can only be called by the original TimerDriver creater with new(). This creates a
    /// copy of the _timer_driver reference and sets a unique id for the child reference.
    ///
    /// # Returns
    ///
    /// A `Result` with `Ok` if all the alarms were handled correctly or an Err(TimerDriverError) if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::OnlyOriginalCopyCanCreateChildren`: if attempting to call this function from a copy which is not the original
    /// - `TimerDriverError::TooManyChildren`: if it too many children are created
    pub(crate) fn create_child_copy(&mut self) -> Result<TimerDriver<'a>, TimerDriverError> {
        if self.id != 0 {
            return Err(TimerDriverError::OnlyOriginalCopyCanCreateChildren);
        }
        let child_id = self.next_child;
        if child_id >= MAX_CHILDREN {
            return Err(TimerDriverError::TooManyChildren);
        }
        self.next_child += 1;
        Ok(Self {
            inner: self.inner.clone(),
            id: child_id,
            next_child: 0,
        })
    }

    /// Async function to sleep on a task
    ///
    /// # Arguments
    /// - mili_secs: Amount of miliseconds for which the task will at least sleep for
    ///
    /// # Returns
    ///
    /// A `Result` with `Ok` if all the delay worked correctly or an Err(TimerDriverError) if it failed
    ///
    /// # Errors
    ///
    /// - `TimerDriverError::ErrorSettingUpForDelay` if the timer driver could not set up in order to execute the delay
    pub async fn delay(&mut self, mili_secs: u32) -> Result<(), TimerDriverError> {
        let notification = Notification::new();
        let notifier = notification.notifier();

        let delay_id = self.id + MAX_CHILDREN;
        self.inner
            .deref_mut()
            .interrupt_after(delay_id, mili_secs as u64 * 1000, move || {
                notifier.notify();
            });
        self.inner
            .deref_mut()
            .enable(delay_id)
            .map_err(|_| TimerDriverError::ErrorSettingUpForDelay)?;

        notification.wait().await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use esp_idf_svc::hal::delay::FreeRtos;

    use crate::microcontroller_src::peripherals::Peripherals;

    use super::*;

    const NEVER: u64 = u64::MAX;

    fn assert_timer_driver_error<T>(
        res: Result<T, TimerDriverError>,
        expected_err: TimerDriverError,
    ) {
        assert_eq!(res.map(|_| "Not an error").unwrap_err(), expected_err)
    }

    fn get_base_timer_driver<'a>() -> (TimerDriver<'a>, Notification) {
        let notif = Notification::new();
        let timer_group = Peripherals::new().get_timer(0);
        (
            TimerDriver::new(timer_group, notif.notifier()).unwrap(),
            notif,
        )
    }

    fn update_after_interrupt<'a>(
        timer_driver: &mut TimerDriver<'a>,
        notif: &Notification,
    ) -> Result<(), Esp32FrameworkError> {
        notif.blocking_wait();
        timer_driver.update_interrupt()
    }

    fn get_2_timer_drivers<'a>() -> (TimerDriver<'a>, TimerDriver<'a>, Notification) {
        let (mut timer_driver, notif) = get_base_timer_driver();
        let child_timer_driver = timer_driver.create_child_copy().unwrap();
        (timer_driver, child_timer_driver, notif)
    }

    #[test]
    fn timer_driver_01_creating_with_timer_group_peripheral() {
        _ = get_base_timer_driver();
    }

    #[test]
    fn timer_driver_02_creating_with_taken_timer_group() {
        let notif = Notification::new();
        let mut peripherals = Peripherals::new();
        let _timer_group = peripherals.get_timer(0);
        let timer_group = peripherals.get_timer(0);
        let res = TimerDriver::new(timer_group, notif.notifier());
        assert_timer_driver_error(
            res,
            TimerDriverError::InvalidPeripheral(PeripheralError::AlreadyTaken),
        );
    }

    #[test]
    fn timer_driver_03_creating_with_non_timer_group_peripheral() {
        let notif = Notification::new();
        let pin = Peripherals::new().get_digital_pin(0);
        let res = TimerDriver::new(pin, notif.notifier());
        assert_timer_driver_error(
            res,
            TimerDriverError::InvalidPeripheral(PeripheralError::NotATimerGroup),
        );
    }

    #[test]
    fn timer_driver_04_notification_is_sent_after_triggering_interrupt() {
        let (mut timer_driver, notif) = get_base_timer_driver();
        timer_driver.interrupt_after(0, || {});
        timer_driver.enable().unwrap();
        assert!(notif.poll())
    }

    #[test]
    fn timer_driver_05_callback_is_executed_after_triggering_interrupt() {
        let (mut timer_driver, notif) = get_base_timer_driver();
        let callback_triggered = SharableRef::new_sharable(false);
        let mut callback_triggered_ref = callback_triggered.clone();
        timer_driver.interrupt_after(0, move || *callback_triggered_ref.deref_mut() = true);
        timer_driver.enable().unwrap();
        update_after_interrupt(&mut timer_driver, &notif).unwrap();
        assert!(*callback_triggered.deref())
    }

    #[test]
    fn timer_driver_06_creating_child() {
        let (mut timer_driver, _) = get_base_timer_driver();
        let child_timer = timer_driver.create_child_copy().unwrap();
        assert_eq!(child_timer.id, timer_driver.id + 1);
        assert_eq!(child_timer.next_child, 0)
    }

    #[test]
    fn timer_driver_07_child_cannot_create_another_child() {
        let (mut timer_driver, _) = get_base_timer_driver();
        let mut child_timer = timer_driver.create_child_copy().unwrap();
        let res = child_timer.create_child_copy();
        assert_timer_driver_error(res, TimerDriverError::OnlyOriginalCopyCanCreateChildren);
    }

    #[test]
    fn timer_driver_08_cannot_create_more_children_than_max_children() {
        let (mut timer_driver, _) = get_base_timer_driver();
        for _ in 0..MAX_CHILDREN - 1 {
            timer_driver.create_child_copy().unwrap();
        }
        let res = timer_driver.create_child_copy();
        assert_timer_driver_error(res, TimerDriverError::TooManyChildren);
    }

    #[test]
    fn timer_driver_09_executing_multiple_simultaneouse_callbacks() {
        let (mut timer_driver0, mut timer_driver1, notif) = get_2_timer_drivers();
        let amount_of_callbacks = SharableRef::new_sharable(0);
        let mut amount_of_callbacks_ref0 = amount_of_callbacks.clone();
        let mut amount_of_callbacks_ref1 = amount_of_callbacks.clone();

        timer_driver0.interrupt_after(0, move || *amount_of_callbacks_ref0.deref_mut() += 1);
        timer_driver1.interrupt_after(0, move || *amount_of_callbacks_ref1.deref_mut() += 1);
        timer_driver0.enable().unwrap();
        timer_driver1.enable().unwrap();

        update_after_interrupt(&mut timer_driver0, &notif).unwrap();
        assert_eq!(*amount_of_callbacks.deref(), 2);
    }

    #[test]
    fn timer_driver_10_executing_multiple_interrupts_set_only_some_trigger() {
        let (mut timer_driver0, mut timer_driver1, notif) = get_2_timer_drivers();
        let amount_of_callbacks = SharableRef::new_sharable(0);
        let mut amount_of_callbacks_ref0 = amount_of_callbacks.clone();
        let mut amount_of_callbacks_ref1 = amount_of_callbacks.clone();

        timer_driver0.interrupt_after(0, move || *amount_of_callbacks_ref0.deref_mut() += 1);
        timer_driver1.interrupt_after(NEVER, move || *amount_of_callbacks_ref1.deref_mut() += 1);
        timer_driver0.enable().unwrap();
        timer_driver1.enable().unwrap();

        update_after_interrupt(&mut timer_driver0, &notif).unwrap();
        assert_eq!(*amount_of_callbacks.deref(), 1);
    }

    #[test]
    fn timer_driver_11_interrupt_multiple_times_first_tick_happens_after_the_time_has_passed() {
        let (mut timer_driver, notif) = get_base_timer_driver();
        let executed_callback = SharableRef::new_sharable(false);
        let mut executed_callback_ref = executed_callback.clone();

        timer_driver.interrupt_after_n_times(NEVER, None, true, move || {
            *executed_callback_ref.deref_mut() = true
        });
        timer_driver.enable().unwrap();

        notif.notifier().notify();
        update_after_interrupt(&mut timer_driver, &notif).unwrap();
        assert!(!*executed_callback.deref());
    }

    #[test]
    fn timer_driver_12_interrupt_multiple_times() {
        let (mut timer_driver, notif) = get_base_timer_driver();
        let amount_of_callbacks = SharableRef::new_sharable(0);
        let mut amount_of_callbacks_ref = amount_of_callbacks.clone();

        timer_driver.interrupt_after_n_times(100, Some(3), true, move || {
            *amount_of_callbacks_ref.deref_mut() += 1
        });
        timer_driver.enable().unwrap();

        for _ in 0..3 {
            update_after_interrupt(&mut timer_driver, &notif).unwrap();
        }

        assert_eq!(*amount_of_callbacks.deref(), 3);
    }

    #[test]
    fn timer_driver_13_interrupt_multiple_times_no_reenable() {
        let (mut timer_driver, notif) = get_base_timer_driver();
        let amount_of_callbacks = SharableRef::new_sharable(0);
        let mut amount_of_callbacks_ref = amount_of_callbacks.clone();

        timer_driver.interrupt_after_n_times(100, Some(3), false, move || {
            *amount_of_callbacks_ref.deref_mut() += 1
        });
        for _ in 0..2 {
            timer_driver.enable().unwrap();
            update_after_interrupt(&mut timer_driver, &notif).unwrap();
        }

        FreeRtos::delay_ms(1);
        notif.notifier().notify();
        update_after_interrupt(&mut timer_driver, &notif).unwrap();

        assert_eq!(*amount_of_callbacks.deref(), 2);
    }

    #[test]
    fn timer_driver_14_remove_interrupt() {
        let (mut timer_driver, notif) = get_base_timer_driver();
        let amount_of_callbacks = SharableRef::new_sharable(0);
        let mut amount_of_callbacks_ref = amount_of_callbacks.clone();

        timer_driver.interrupt_after_n_times(100, Some(3), false, move || {
            *amount_of_callbacks_ref.deref_mut() += 1
        });
        timer_driver.enable().unwrap();

        update_after_interrupt(&mut timer_driver, &notif).unwrap();
        timer_driver.enable().unwrap();

        timer_driver.remove_interrupt().unwrap();
        FreeRtos::delay_ms(1);
        notif.notifier().notify();
        update_after_interrupt(&mut timer_driver, &notif).unwrap();

        assert_eq!(*amount_of_callbacks.deref(), 1);
    }
}
