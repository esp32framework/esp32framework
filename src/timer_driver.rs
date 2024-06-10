use esp_idf_svc::hal::timer;
use crate::timer_driver::timer::TimerConfig;
use esp_idf_svc::hal::peripheral::Peripheral;

pub struct TimerDriver<'a> {
    driver: timer::TimerDriver<'a>
}

#[derive(Debug)]
pub enum TimerDriverError {
    CouldNotSetTimer,
    InvalidTimer,
    CannotSetTimerCounter,
    SubscriptionError
}


impl <'a>TimerDriver<'a>{
    pub fn new<T: timer::Timer>(timer: impl Peripheral<P = T> + 'a)->Result<TimerDriver<'a>, TimerDriverError> {
        let driver = timer::TimerDriver::new(timer, &TimerConfig::new()).map_err(|_| TimerDriverError::InvalidTimer)?;
        Ok(TimerDriver{driver})
    }
    
    pub fn interrupt_after<F: FnMut() + Send + 'static>(&mut self, micro_seconds: u32, callback: F)-> Result<(), TimerDriverError>{
        unsafe{
            self.driver.subscribe(callback).map_err(|_| TimerDriverError::SubscriptionError)?;
        }
        self.driver.set_alarm(((micro_seconds as u64) * self.driver.tick_hz()/1000000) as u64).map_err(|_| TimerDriverError::CouldNotSetTimer)
    }

    fn _enable(&mut self, enable: bool) -> Result<(),TimerDriverError>{
        if enable{
            self.driver.set_counter(0).map_err(|_| TimerDriverError::CannotSetTimerCounter)?; 
            self.driver.enable_interrupt().map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }else{
            self.driver.disable_interrupt().map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        }
        self.driver.enable_alarm(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        self.driver.enable(enable).map_err(|_| TimerDriverError::CouldNotSetTimer)?;
        Ok(())   
    }

    pub fn enable(&mut self) -> Result<(),TimerDriverError>{
        self._enable(true)
    }
    
    pub fn disable(&mut self) -> Result<(),TimerDriverError>{
        self._enable(false)
    }

    pub fn unsubscribe(&mut self)  -> Result<(),TimerDriverError> {
        self.driver.unsubscribe().map_err(|_| TimerDriverError::SubscriptionError)
    }
}
