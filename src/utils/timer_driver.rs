use std::{
    cell::RefCell, collections::{BinaryHeap, HashMap}, num::NonZeroU32, rc::Rc, sync::{
        atomic::{AtomicBool, Ordering},
        Arc
    }
};

use esp_idf_svc::hal::timer;
use crate::{microcontroller_src::interrupt_driver::InterruptDriver, utils::timer_driver::timer::TimerConfig};
use crate::microcontroller_src::peripherals::Peripheral;
use sharable_reference_macro::sharable_reference_wrapper;

use super::{auxiliary::{SharableRef, SharableRefExt}, esp32_framework_error::Esp32FrameworkError, notification::{self, Notification, Notifier}};

const MICRO_IN_SEC: u64 = 1000000;
const MAX_CHILDREN: u16 = u8::MAX as u16;

/// Wrapper of _TimerDriver, handling the coordination of multiple references to the inner driver, 
/// in order to allow for interrupts to be set per timer resource. Each reference has a unique id
/// and can create one interrupt each.
/// In order to see the documentation of wrapper functions see [_TimerDriver]
pub struct TimerDriver<'a> {
    inner: SharableRef<_TimerDriver<'a>>,
    id: u16,
    next_child: u16,
}

/// Driver for handling the timer resource, allowing for multiple interrupts to be set
struct _TimerDriver<'a> {
    driver: timer::TimerDriver<'a>,
    interrupt_update: InterruptUpdate,
    alarms: BinaryHeap<Alarm>,
    interrupts: HashMap<u16, TimeInterrupt>,
}

#[derive(Debug, Clone, Copy)]
pub enum TimerDriverError {
    ErrorReadingTimer,
    ErrorReadingAlarm,
    CouldNotSetTimer,
    InvalidTimer,
    CannotSetTimerCounter,
    SubscriptionError,
    TooManyChildren,
    OnlyOriginalCopyCanCreateChildren
}

/// Represents an interrupt to be executed after some time a number of times
struct TimeInterrupt{
    after: u64,
    id: u16,
    current_alarm_id: usize,
    status: TimerInterruptStatus,
    remaining_triggers: Option<u32>,
    auto_reenable: bool,
    callback: Box<dyn FnMut()>
}

#[derive(Debug, PartialEq, Eq)]
enum TimerInterruptStatus{
    Enabled,
    Disabled,
    Removing
}

#[derive(Debug, PartialEq, Eq)]
struct Alarm{
    time: u64,
    id: u16,
    alarm_id: usize
}

/// After an interrupt is triggered an InterruptUpdate will be set and handled
#[derive(Clone)]
struct InterruptUpdate{
    update: Arc<AtomicBool>
}

impl InterruptUpdate{
    fn new()->InterruptUpdate{
        InterruptUpdate{update: Arc::new(AtomicBool::new(true))}
    }
    
    /// Checks for an update
    fn any_updates(&self)->bool {
        self.update.load(Ordering::Relaxed)
    }
    
    /// Sets an update on the interrupt update
    fn new_update(&self){
        self.update.store(true, Ordering::Relaxed);
    }
    
    /// Removes update
    fn handling_update(&self){
        self.update.store(false, Ordering::Relaxed);
    }
    
    /// If there are any updates it handles them
    fn handle_any_updates(&self)->bool{
        if self.any_updates(){
            self.handling_update();
            true
        }else{
            false
        }
    }
}

impl TimeInterrupt{
    fn new(id:u16, callback: Box<dyn FnMut()>, time: u64, amount_of_triggers: Option<u32>, auto_reenable: bool)-> TimeInterrupt{
        TimeInterrupt{
            after: time,
            id,
            current_alarm_id: 0,
            status: TimerInterruptStatus::Disabled,
            remaining_triggers: amount_of_triggers,
            auto_reenable,
            callback,
        }
    }

    fn get_alarm(&self, current_time: u64)-> Alarm{
        Alarm::new(self.id, self.current_alarm_id, self.after + current_time)
    }

    /// Makes it so all previouse alarms are ignored, by advancing the alarm id
    fn disable_previouse_alarms(&mut self){
        self.current_alarm_id += 1
    }

    /// If any triggers remain execute the callback
    fn trigger(&mut self){
        if let Some(ref mut amount) = self.remaining_triggers{
            if *amount == 0{
                return
            }
            *amount-=1;
        }
        (self.callback)();
        self.status = TimerInterruptStatus::Disabled;
    }

    /// Checks if there are any triggers left or there was no limit set to the amount of triggers
    fn any_triggers_left(&self)-> bool{
        match self.remaining_triggers{
            Some(triggers) => triggers > 0,
            None => true,
        }
    }
}

impl Alarm{
    fn new(id: u16, alarm_id: usize, time: u64)-> Self{
        Alarm { time, id, alarm_id}
    }
}

impl Ord for Alarm{
    // Order is inverted for insertion as minimal heap
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.time.cmp(&other.time){
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

impl PartialOrd for Alarm{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[sharable_reference_wrapper("id")]
impl <'a>_TimerDriver<'a>{
    fn new(timer: Peripheral, notifier: Notifier) -> Result<_TimerDriver<'a>, TimerDriverError> {
        let driver = match timer{
            Peripheral::Timer(timer_num) => 
                match timer_num{
                    0 => timer::TimerDriver::new(unsafe{timer::TIMER00::new()}, &TimerConfig::new()),
                    1 => timer::TimerDriver::new(unsafe{timer::TIMER10::new()}, &TimerConfig::new()),
                    _ => return Err(TimerDriverError::InvalidTimer),
                }.map_err(|_| TimerDriverError::InvalidTimer)?,
            _ => return Err(TimerDriverError::InvalidTimer),
        };

        let mut timer = _TimerDriver{
            driver, 
            interrupt_update: InterruptUpdate::new(),
            alarms: BinaryHeap::new(),
            interrupts: HashMap::new(),
        };
        timer.set_interrupt_update_callback(notifier).map(|_| timer)
    }
    
    /// Sets the callback for the timer_driver which will modify the interrupt update
    fn set_interrupt_update_callback(&mut self, notifier: Notifier)->Result<(), TimerDriverError>{
        let interrupt_update_ref = self.interrupt_update.clone();
        unsafe{
            let alarm_callback = move || {
                interrupt_update_ref.new_update();
                notifier.notify().unwrap();
            };
        
            self.driver.subscribe(alarm_callback).map_err(|_| TimerDriverError::SubscriptionError)
        }
    }

    /// Sets an interrupt that triggers once after "microseconds". For this to start working enable()
    /// must be called. After the interrupt has been trigger it can be reset by calling enable()
    pub fn interrupt_after<F: FnMut() + 'static>(&mut self, id: u16, micro_seconds: u64, callback: F){
        self.interrupt_after_n_times(id, micro_seconds, None, false, callback)
    }

    /// Sets an interrupt to trigger every "micro_seconds" for an "amount_of_times" if given, if not
    /// triggers indefinitly. If autoenable is set, after triggering the callback, it will be set again
    /// if not it will have to be reenabled manually by caling enable(). For this to start working 
    /// enable() must be called. There can only be one callback per id.
    pub fn interrupt_after_n_times<F: FnMut() + 'static>(&mut self, id: u16, micro_seconds: u64, amount_of_triggers: Option<u32>, auto_reenable: bool, callback: F){        
        let time = self.micro_to_counter(micro_seconds);
        let interrupt = TimeInterrupt::new(id, Box::new(callback), time, amount_of_triggers, auto_reenable);
        if let Some(old_interrupt) = self.interrupts.insert(id, interrupt){
            self.interrupts.get_mut(&id).unwrap().current_alarm_id = old_interrupt.current_alarm_id + 1
        }
    }

    /// transforms microseconds to the microcontroller tick_hz
    fn micro_to_counter(&self, micro_seconds: u64)->u64{
        micro_seconds * self.driver.tick_hz() / MICRO_IN_SEC
    }

    /// Activates the timeInterrupt corresponding to "id".
    fn activate(&mut self, id: u16)-> Result<(), TimerDriverError>{
        if let Some(interrupt) = self.interrupts.get_mut(&id){
            if interrupt.status == TimerInterruptStatus::Disabled{
                let current_time = self.driver.counter().map_err(|_| TimerDriverError::ErrorReadingTimer)?;
                self.alarms.push(interrupt.get_alarm(current_time))
            }
            interrupt.status = TimerInterruptStatus::Enabled
        }
        Ok(())
    }
    
    /// Diactivates the timeInterrupt corresponding to "id".
    fn diactivate(&mut self, id: u16){
        if let Some(interrupt) = self.interrupts.get_mut(&id){
            if interrupt.status == TimerInterruptStatus::Enabled{
                interrupt.disable_previouse_alarms()
            }
            interrupt.status = TimerInterruptStatus::Disabled
        }
    }

    fn reset(&mut self){
        self.interrupts = HashMap::new();
        self.interrupt_update.handling_update();
        self.alarms = BinaryHeap::new();
    }

    /// Enables or disables the interrupt corresponding to "id". If the interrupt is enabled, if it 
    /// is the new lowest time, the alarm is updated. When the first interrupt is enabled, or the last
    /// disabled the timer is stoped
    fn _enable(&mut self, id: u16, enable: bool) -> Result<(),TimerDriverError>{
        let starting_len = self.alarms.len();
        if enable{
            self.activate(id)?;
            self.set_lowest_alarm()?;
        }else{
            self.diactivate(id);
        }
        
        if self.alarms.is_empty() || starting_len == 0{
            if enable{
                self.driver.enable_interrupt().map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            }else{
                self.driver.disable_interrupt().map_err(|_| TimerDriverError::CouldNotSetTimer)?;
                self.reset()
            }
            self.driver.enable_alarm(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            self.driver.enable(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }
        Ok(())
    }

    /// Enables the interrupt corresponding to "id". If the interrupt is enabled, if it 
    /// is the new lowest time, the alarm is updated. When the first interrupt is enabled,
    /// the timer is stoped
    pub fn enable(&mut self, id: u16) -> Result<(),TimerDriverError>{
        self._enable(id, true)
    }
    
    /// Disables the interrupt corresponding to "id". When the last disabled the timer is stoped
    pub fn disable(&mut self, id: u16) -> Result<(),TimerDriverError>{
        self._enable(id, false)
    }
    
    /// Removes the interrupt corresponding to "id"
    pub fn remove_interrupt(&mut self, id:u16)->Result<(), TimerDriverError>{
        self.disable(id)?;
        if let Some(interrupt) = self.interrupts.get_mut(&id){
            interrupt.status = TimerInterruptStatus::Removing;
        }
        Ok(())
    }

    /// Sets the interrupt to trigger on the soonest alarm
    fn set_lowest_alarm(&mut self)-> Result<(),TimerDriverError> {
        if let Some(alarm) = self.alarms.peek(){
            if alarm.time != self.driver.alarm().map_err(|_| TimerDriverError::ErrorReadingAlarm)?{
                self.driver.set_alarm(alarm.time).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            }
            self.driver.enable_alarm(true).map_err(|_| TimerDriverError::ErrorReadingTimer)?;
        }
        Ok(())
    }

    fn handle_alarm_update(&mut self, alarm: Alarm) -> Result<(), TimerDriverError>{
        if let Some(interrupt) = self.interrupts.get_mut(&alarm.id){
            if interrupt.current_alarm_id == alarm.alarm_id{
                match interrupt.status{
                    TimerInterruptStatus::Enabled => {
                        interrupt.trigger();
                        if interrupt.any_triggers_left() && interrupt.auto_reenable{
                            self.activate(alarm.id)?;
                        }
                    },
                    TimerInterruptStatus::Disabled => {},
                    TimerInterruptStatus::Removing => {self.interrupts.remove(&alarm.id);},
                }
            }
        }
        Ok(())
    }

    /// Handles the diferent type of interrupts and reenabling the interrupt when necesary
    fn _update_interrupt(&mut self)-> Result<(), TimerDriverError> {  
        let mut updates = self.interrupt_update.handle_any_updates();
        while updates{
            if let Some(alarm) = self.alarms.pop(){
                self.handle_alarm_update(alarm)?;
            }
            self.set_lowest_alarm()?;
            updates = self.interrupt_update.handle_any_updates();
        }
        Ok(())
    }
}

#[sharable_reference_wrapper("id")]
impl <'a> InterruptDriver for _TimerDriver<'a>{
    fn update_interrupt(&mut self)-> Result<(), Esp32FrameworkError> {
        self._update_interrupt().map_err(Esp32FrameworkError::TimerDriver)
    }
}

impl <'a>TimerDriver<'a>{
    pub fn new(timer: Peripheral, notifier: Notifier) -> Result<TimerDriver<'a>, TimerDriverError> {
        Ok(TimerDriver{
            inner: SharableRef::new_sharable(_TimerDriver::new(timer, notifier)?),
            id: 0,
            next_child: 1,
        })
    }

    /// This function can only be called by the original TimerDriver creater with new(). This reates a 
    /// copy of the _timer_driver reference and sets a unique id for the child reference.
    pub fn create_child_copy(&mut self) -> Result<TimerDriver<'a>, TimerDriverError>{
        if self.id != 0{
            return Err(TimerDriverError::OnlyOriginalCopyCanCreateChildren)
        }
        let child_id = self.next_child;
        if child_id >= MAX_CHILDREN{
            return Err(TimerDriverError::TooManyChildren)
        }
        self.next_child += 1;
        Ok(Self{
            inner:self.inner.clone(),
            id: child_id,
            next_child: 0,
        })
    }

    pub async fn delay(&mut self, mili_secs: u32) -> Result<(), TimerDriverError>{
        let notification = Notification::new();
        let notifier = notification.notifier();

        let delay_id = self.id + MAX_CHILDREN;
        self.inner.deref_mut().interrupt_after(delay_id, mili_secs as u64 * 1000, move ||{
            notifier.notify().unwrap();
        });
        self.inner.deref_mut().enable(delay_id).unwrap();

        notification.wait().await;
        Ok(())
    }
}