use std::{
    cell::RefCell, collections::{BinaryHeap, HashMap, HashSet}, rc::Rc, sync::{
        atomic::{AtomicBool, AtomicU8, Ordering},
        Arc
    }
};

use esp_idf_svc::hal::timer;
use crate::utils::timer_driver::timer::TimerConfig;
use crate::microcontroller::peripherals::Peripheral;


const MICRO_IN_SEC: u64 = 1000000;

pub struct TimerDriver<'a> {
    inner: Rc<RefCell<_TimerDriver<'a>>>,
    id: u8,
    next_child: u8,
}

struct _TimerDriver<'a> {
    driver: timer::TimerDriver<'a>,
    interrupt_update: InterruptUpdate,
    interrupts: BinaryHeap<TimeInterrupt>,
    inactive_alarms: HashMap<u8, DisabledTimeInterrupt>
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

/// After an interrupt is triggered an InterruptUpdate will be set and handled
#[derive(Clone)]
struct InterruptUpdate{
    update: Arc<AtomicBool>
}

impl InterruptUpdate{
    fn new()->InterruptUpdate{
        InterruptUpdate{update: Arc::new(AtomicBool::new(true))}
    }

    fn any_updates(&self)->bool {
        self.update.load(Ordering::Relaxed)
    }
    
    fn new_update(&self){
        self.update.store(true, Ordering::Relaxed);
    }
    
    fn handling_update(&self){
        self.update.store(false, Ordering::Relaxed);
    }

    fn handle_any_updates(&self)->bool{
        if self.any_updates(){
            self.handling_update();
            true
        }else{
            false
        }
    }
}

struct TimeInterrupt{
    after: u64,
    alarm_time: u64,
    id: u8,
    remaining_triggers: Option<u32>,
    callback: Box<dyn FnMut()>
}

enum DisabledTimeInterrupt{
    Interrupt(TimeInterrupt),
    Waiting,
    Removing
}

impl TimeInterrupt{
    fn new(id:u8, callback: Box<dyn FnMut()>, time: u64, amount_of_triggers: Option<u32>)-> TimeInterrupt{
        TimeInterrupt{
            after: time,
            alarm_time: 0,
            id: id,
            remaining_triggers: amount_of_triggers,
            callback,
        }
    }

    fn trigger(&mut self){
        if let Some(ref mut amount) = self.remaining_triggers{
            if *amount <= 0{
                return
            }
            *amount-=1;
        }
        (self.callback)()
    }

    fn any_triggers_left(&self)-> bool{
        match self.remaining_triggers{
            Some(triggers) => triggers > 0,
            None => true,
        }
    }
}

impl Ord for TimeInterrupt{
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

impl <'a>_TimerDriver<'a>{
    fn new(timer: Peripheral) -> Result<_TimerDriver<'a>, TimerDriverError> {
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
            inactive_alarms: HashMap::new()
        };
        timer.set_interrupt_update_callback().map(|_| timer)
    }
    
    fn set_interrupt_update_callback(&mut self)->Result<(), TimerDriverError>{
        let interrupt_update_ref = self.interrupt_update.clone();
        let alarm_callback = move || {
            interrupt_update_ref.new_update()
        };
        
        unsafe{
            self.driver.subscribe(alarm_callback).map_err(|_| TimerDriverError::SubscriptionError)
        }
    }

    fn interrupt_after<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u64, callback: F)-> Result<(), TimerDriverError>{
        unsafe{
            self.driver.subscribe(callback).map_err(|_| TimerDriverError::SubscriptionError)?;
        }
        self.driver.set_alarm(((micro_seconds as u64) * self.driver.tick_hz()/1000000) as u64).map_err(|_| TimerDriverError::CouldNotSetTimer)
    }
    
    fn interrupt_after_n_times<F: FnMut() + Send + 'static>(&mut self, id: u8, micro_seconds: u64, amount_of_triggers: Option<u32>, callback: F){        
        let time = self.micro_to_counter(micro_seconds);
        let alarm = TimeInterrupt::new(id, Box::new(callback), time, amount_of_triggers);
        self.inactive_alarms.insert(alarm.id, DisabledTimeInterrupt::Interrupt(alarm));
    }

    fn micro_to_counter(&self, micro_seconds: u64)->u64{
        micro_seconds * self.driver.tick_hz() / MICRO_IN_SEC
    }

    fn add_time_interrupt(&mut self, mut time_interrupt: TimeInterrupt)-> Result<(), TimerDriverError>{
        time_interrupt.alarm_time = self.driver.counter().map_err(|_| TimerDriverError::ErrorReadingTimer)? + time_interrupt.after;
        self.interrupts.push(time_interrupt);
        Ok(())
    }

    fn activate(&mut self, id: u8)-> Result<(), TimerDriverError>{
        if let Some(DisabledTimeInterrupt::Interrupt(time_interrupt)) = self.inactive_alarms.remove(&id){
            self.add_time_interrupt(time_interrupt)?
        }
        Ok(())
    }

    fn diactivate(&mut self, id: u8){
        self.inactive_alarms.insert(id, DisabledTimeInterrupt::Waiting);
    }

    fn reset(&mut self){
        self.inactive_alarms = HashMap::new();
        self.interrupt_update.handling_update();
        self.interrupts = BinaryHeap::new();
    }

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
            self.driver.set_counter(0).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            self.driver.enable_alarm(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
            self.driver.enable(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }
        Ok(())
    }

    fn enable(&mut self, id: u8) -> Result<(),TimerDriverError>{
        self._enable(id, true)
    }
    
    fn disable(&mut self, id: u8) -> Result<(),TimerDriverError>{
        self._enable(id, false)
    }

    fn remove_interrupt(&mut self, id:u8)->Result<(), TimerDriverError>{
        self.disable(id)?;
        if self.inactive_alarms.contains_key(&id){
            self.inactive_alarms.insert(id, DisabledTimeInterrupt::Removing);
        }
        Ok(())
    }

    fn unsubscribe(&mut self)  -> Result<(),TimerDriverError> {
        self.driver.unsubscribe().map_err(|_| TimerDriverError::SubscriptionError)
    }

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
                    Some(disabled) => {match disabled{
                        DisabledTimeInterrupt::Removing => self.inactive_alarms.remove(&interrupt_update.id),
                        _ => self.inactive_alarms.insert(interrupt_update.id, DisabledTimeInterrupt::Interrupt(interrupt_update)),
                    };},
                    None => {
                        interrupt_update.trigger();
                        if interrupt_update.any_triggers_left(){
                            self.add_time_interrupt(interrupt_update)?;
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
    //pub fn new<T: timer::Timer>(timer: impl Peripheral<P = T> + 'a)->Result<TimerDriver<'a>, TimerDriverError> {
    pub fn new(timer: Peripheral) -> Result<TimerDriver<'a>, TimerDriverError> {
        Ok(TimerDriver{
            inner: Rc::new(RefCell::new(_TimerDriver::new(timer)?)),
            id: 0,
            next_child: 1,
        })
    }
    
    pub fn interrupt_after<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u64, callback: F)-> Result<(), TimerDriverError>{
        self.inner.borrow_mut().interrupt_after(micro_seconds, callback)
    }
    
    pub fn interrupt_after_n_times<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u64, amount_of_triggers: Option<u32>, callback: F){
        self.inner.borrow_mut().interrupt_after_n_times(self.id, micro_seconds, amount_of_triggers, callback)
    }

    pub fn enable(&mut self) -> Result<(),TimerDriverError>{
        self.inner.borrow_mut().enable(self.id)
    }
    
    pub fn disable(&mut self) -> Result<(),TimerDriverError>{
        self.inner.borrow_mut().disable(self.id)
    }
    
    pub fn remove_interrupt(&mut self)-> Result<(), TimerDriverError>{
        self.inner.borrow_mut().remove_interrupt(self.id)
    }

    // TODO ver de borrar
    pub fn unsubscribe(&mut self)  -> Result<(),TimerDriverError> {
        self.inner.borrow_mut().unsubscribe()
    }

    pub fn update_interrupts(&mut self) -> Result<(), TimerDriverError> {
        self.inner.borrow_mut().update_interrupts()
    }

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

