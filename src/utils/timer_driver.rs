use std::{
    cell::RefCell, collections::{BinaryHeap, HashMap}, num::NonZeroU32, rc::Rc, sync::{
        atomic::{AtomicBool, Ordering},
        Arc
    }
};

use esp_idf_svc::hal::{task::notification::Notifier, timer};
use crate::utils::timer_driver::timer::TimerConfig;
use crate::microcontroller::peripherals::Peripheral;
use sharable_reference_macro::sharable_reference_wrapper;
const MICRO_IN_SEC: u64 = 1000000;

/// Wrapper of _TimerDriver, handling the coordination of multiple references to the inner driver, 
/// in order to allow for interrupts to be set per timer resource. Each reference has a unique id
/// and can create one interrupt each.
/// In order to see the documentation of wrapper functions see [_TimerDriver]
pub struct TimerDriver<'a> {
    inner: Rc<RefCell<_TimerDriver<'a>>>,
    id: u8,
    next_child: u8,
}

/// Driver for handling the timer resource, allowing for multiple interrupts to be set
struct _TimerDriver<'a> {
    driver: timer::TimerDriver<'a>,
    interrupt_update: InterruptUpdate,
    interrupts: BinaryHeap<TimeInterrupt>,
    inactive_alarms: HashMap<u8, DisabledTimeInterrupt>,
}

#[derive(Debug)]
pub enum TimerDriverError {
    ErrorReadingTimer,
    ErrorReadingAlarm,
    CouldNotSetTimer,
    InvalidTimer,
    CannotSetTimerCounter,
    SubscriptionError,
    OnlyOriginalCopyCanCreateChildren
}

/// Represents an interrupt to be executed after some time a number of times
struct TimeInterrupt{
    after: u64,
    alarm_time: u64,
    id: u8,
    remaining_triggers: Option<u32>,
    auto_reenable: bool,
    callback: Box<dyn FnMut()>
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


/// Diferent States on which a disabledInterrupt can be:
///     - DisabledTimeInterrupt::Interrupt holds the TimeInterrupt waiting to be reenabled
///     - Waiting means the corresponding TimeInterrupt is still in the active interrupts of the timer 
///       driver, but will be added on a subsequent call to interrupt update
///     - Removing means the corresponding TimeInterrupt is still in the active interrupts of the timer
///       driver, but will be removed on a subsequent call to interrupt update
enum DisabledTimeInterrupt{
    Interrupt(TimeInterrupt),
    Waiting,
    Removing
}

impl TimeInterrupt{
    fn new(id:u8, callback: Box<dyn FnMut()>, time: u64, amount_of_triggers: Option<u32>, auto_reenable: bool)-> TimeInterrupt{
        TimeInterrupt{
            after: time,
            alarm_time: 0,
            id: id,
            remaining_triggers: amount_of_triggers,
            auto_reenable: auto_reenable,
            callback,
        }
    }

    /// If any triggers remain execute the callback
    fn trigger(&mut self){
        if let Some(ref mut amount) = self.remaining_triggers{
            if *amount <= 0{
                return
            }
            *amount-=1;
        }
        (self.callback)()
    }

    /// Checks if there are any triggers left or there was no limit set to the amount of triggers
    fn any_triggers_left(&self)-> bool{
        match self.remaining_triggers{
            Some(triggers) => triggers > 0,
            None => true,
        }
    }
}

impl Ord for TimeInterrupt{
    // Order is inverted for insertion as minimal heap
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.alarm_time.cmp(&other.alarm_time){
            std::cmp::Ordering::Less => std::cmp::Ordering::Greater,
            std::cmp::Ordering::Equal => std::cmp::Ordering::Equal,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Less,
        }
    }
}

impl PartialOrd for TimeInterrupt{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for TimeInterrupt{
    fn eq(&self, other: &Self) -> bool {
        self.after == other.after && self.id == other.id
    }
}

impl Eq for TimeInterrupt{}

#[sharable_reference_wrapper("id")]
impl <'a>_TimerDriver<'a>{
    fn new(timer: Peripheral, notifier: Arc<Notifier>) -> Result<_TimerDriver<'a>, TimerDriverError> {
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
            interrupts: BinaryHeap::new(),
            inactive_alarms: HashMap::new(),
        };
        timer.set_interrupt_update_callback(notifier).map(|_| timer)
    }
    
    /// Sets the callback for the timer_driver which will modify the interrupt update
    fn set_interrupt_update_callback(&mut self, notifier: Arc<Notifier>)->Result<(), TimerDriverError>{
        let interrupt_update_ref = self.interrupt_update.clone();
        unsafe{
            let alarm_callback = move || {
                interrupt_update_ref.new_update();
                notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
            };
        
            self.driver.subscribe(alarm_callback).map_err(|_| TimerDriverError::SubscriptionError)
        }
    }

    /// Sets an interrupt that triggers once after "microseconds". For this to start working enable()
    /// must be called. After the interrupt has been trigger it can be reset by calling enable()
    pub fn interrupt_after<F: FnMut() + Send + 'static>(&mut self, id: u8, micro_seconds: u64, callback: F){
        self.interrupt_after_n_times(id, micro_seconds, None, false, callback)
    }

    /// Sets an interrupt to trigger every "micro_seconds" for an "amount_of_times" if given, if not
    /// triggers indefinitly. If autoenable is set, after triggering the callback, it will be set again
    /// if not it will have to be reenabled manually by caling enable(). For this to start working 
    /// enable() must be called. There can only be one callback per id.
    pub fn interrupt_after_n_times<F: FnMut() + Send + 'static>(&mut self, id: u8, micro_seconds: u64, amount_of_triggers: Option<u32>, auto_reenable: bool, callback: F){        
        let time = self.micro_to_counter(micro_seconds);
        let alarm = TimeInterrupt::new(id, Box::new(callback), time, amount_of_triggers, auto_reenable);
        self.inactive_alarms.insert(alarm.id, DisabledTimeInterrupt::Interrupt(alarm));
    }

    /// transforms microseconds to the microcontroller tick_hz
    fn micro_to_counter(&self, micro_seconds: u64)->u64{
        micro_seconds * self.driver.tick_hz() / MICRO_IN_SEC
    }

    /// Sets the alarm time for an interrupt update and adds it to the timer_driver
    fn add_active_time_interrupt(&mut self, mut time_interrupt: TimeInterrupt)-> Result<(), TimerDriverError>{
        time_interrupt.alarm_time = self.driver.counter().map_err(|_| TimerDriverError::ErrorReadingTimer)? + time_interrupt.after;
        self.interrupts.push(time_interrupt);
        Ok(())
    }

    /// Activates the timeInterrupt corresponding to "id".
    fn activate(&mut self, id: u8)-> Result<(), TimerDriverError>{
        if let Some(DisabledTimeInterrupt::Interrupt(time_interrupt)) = self.inactive_alarms.remove(&id){
            self.add_active_time_interrupt(time_interrupt)?
        }
        Ok(())
    }

    /// Diactivates the timeInterrupt corresponding to "id". This is done by setting the id as Waiting.
    fn diactivate(&mut self, id: u8){
        for interrupt in &self.interrupts{
            if id == interrupt.id{
                if !self.inactive_alarms.contains_key(&id){
                    self.inactive_alarms.insert(id, DisabledTimeInterrupt::Waiting);
                }
            }
        }
    }

    fn reset(&mut self){
        self.inactive_alarms = HashMap::new();
        self.interrupt_update.handling_update();
        self.interrupts = BinaryHeap::new();
    }

    /// Enables or disables the interrupt corresponding to "id". If the interrupt is enabled, if it 
    /// is the new lowest time, the alarm is updated. When the first interrupt is enabled, or the last
    /// disabled the timer is stoped
    fn _enable(&mut self, id: u8, enable: bool) -> Result<(),TimerDriverError>{
        let starting_len = self.interrupts.len();
        if enable{
            self.activate(id)?;
            self.set_lowest_alarm()?;
        }else{
            self.diactivate(id);
        }
        
        if self.interrupts.len() == 0 || starting_len == 0{
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
    pub fn enable(&mut self, id: u8) -> Result<(),TimerDriverError>{
        self._enable(id, true)
    }
    
    /// Disables the interrupt corresponding to "id". When the last disabled the timer is stoped
    pub fn disable(&mut self, id: u8) -> Result<(),TimerDriverError>{
        self._enable(id, false)
    }
    
    /// Removes the interrupt corresponding to "id"
    pub fn remove_interrupt(&mut self, id:u8)->Result<(), TimerDriverError>{
        self.disable(id)?;
        if self.inactive_alarms.contains_key(&id){
            self.inactive_alarms.insert(id, DisabledTimeInterrupt::Removing);
        }
        Ok(())
    }

    /// Sets the interrupt to trigger on the soonest alarm
    fn set_lowest_alarm(&mut self)-> Result<(),TimerDriverError> {
        if let Some(interrupt) = self.interrupts.peek(){
            if interrupt.alarm_time != self.driver.alarm().map_err(|_| TimerDriverError::ErrorReadingAlarm)?{
                self.driver.set_alarm(interrupt.alarm_time).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            }
            self.driver.enable_alarm(true).map_err(|_| TimerDriverError::ErrorReadingTimer)?;
        }
        Ok(())
    }

    /// Handles the diferent type of interrupts and reenabling the interrupt when necesary
    pub fn update_interrupts(&mut self) -> Result<(), TimerDriverError> {
        let mut updates = self.interrupt_update.handle_any_updates();
        while updates{
            if let Some(mut interrupt_update) = self.interrupts.pop(){
                match self.inactive_alarms.get_mut(&interrupt_update.id){
                    Some(disabled) => {
                        match disabled{
                            DisabledTimeInterrupt::Removing => self.inactive_alarms.remove(&interrupt_update.id),
                            _ => self.inactive_alarms.insert(interrupt_update.id, DisabledTimeInterrupt::Interrupt(interrupt_update)),
                        };
                    },
                    None => {
                        interrupt_update.trigger();
                        if interrupt_update.any_triggers_left(){
                            if interrupt_update.auto_reenable{
                                self.add_active_time_interrupt(interrupt_update)?;
                            }else{
                                self.inactive_alarms.insert(interrupt_update.id, DisabledTimeInterrupt::Interrupt(interrupt_update));
                            }
                        }
                    },
                };
            }
            self.set_lowest_alarm()?;
            updates = self.interrupt_update.handle_any_updates();
        }
        Ok(())
    }
}

impl <'a>TimerDriver<'a>{
    pub fn new(timer: Peripheral, notifier: Arc<Notifier>) -> Result<TimerDriver<'a>, TimerDriverError> {
        Ok(TimerDriver{
            inner: Rc::new(RefCell::new(_TimerDriver::new(timer, notifier)?)),
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
        self.next_child += 1;
        Ok(Self{
            inner:self.inner.clone(),
            id: child_id,
            next_child: 0,
        })
    }
}

