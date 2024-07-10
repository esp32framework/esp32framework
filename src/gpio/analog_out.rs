use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::sys::ESP_FAIL;
use crate::microcontroller::peripherals::Peripheral;
use crate::utils::timer_driver::TimerDriver;
use crate::utils::timer_driver::TimerDriverError;
use std::sync::atomic::AtomicBool;
use std::sync::{
    Arc,
    atomic::{AtomicU8, AtomicU32, Ordering}
};

type AtomicInterruptUpdateCode = AtomicU8;

#[derive(Debug)]
pub enum AnalogOutError{
    TooManyPWMOutputs,
    InvalidArg,
    InvalidPeripheral,
    InvalidFrequencyOrDuty,
    ErrorSettingOutput,
    TimerDriverError(TimerDriverError)
}

enum FixedChangeType{
    Increase,
    Decrease,
    IncreaseDecrease,
    DecreaseIncrease,
    None
}

/// Driver to handle an analog output for a particular pin
pub struct AnalogOut<'a> {
    driver: LedcDriver<'a>,
    timer_driver: TimerDriver<'a>,
    duty: Arc<AtomicU32>,
    interrupt_update_code: Arc<AtomicInterruptUpdateCode>,
    fixed_change_increasing: Arc<AtomicBool>,
    fixed_change_type: FixedChangeType,
}

pub enum InterruptUpdate {
    ChangeDuty,
    None
}

impl InterruptUpdate{
    fn get_code(self)-> u8{
        self as u8
    }

    fn get_atomic_code(self)-> AtomicInterruptUpdateCode{
        AtomicInterruptUpdateCode::new(self.get_code())
    }

    fn from_code(code:u8)-> Self {
        match code{
            0 => InterruptUpdate::ChangeDuty,
            _ => Self::None,
        }
    }

    fn from_atomic_code(atomic_code: Arc<AtomicInterruptUpdateCode>) -> Self {
        InterruptUpdate::from_code(atomic_code.load(Ordering::Acquire))
    }
}

impl <'a>AnalogOut<'a> {
    //TODO: Dejar elegir al usuario low y high resolution, segun que timer
    
    /// Creates a new AnalogOut from a pin number, frequency and resolution.
    pub fn new(peripheral_channel: Peripheral, timer:Peripheral, gpio_pin: Peripheral, timer_driver: TimerDriver<'a>, freq_hz: u32, resolution: u32) -> Result<AnalogOut<'a>, AnalogOutError> {
        let resolution = AnalogOut::create_resolution(resolution);
        let config = &config::TimerConfig::new().frequency(freq_hz.Hz().into()).resolution(resolution);
        AnalogOut::_new(peripheral_channel, timer, gpio_pin, timer_driver, config)
    }
    
    /// Creates a new AnalogOut for a specific pin with a given configuration of frecuency and resolution.
    pub fn _new(peripheral_channel: Peripheral, timer:Peripheral, gpio_pin: Peripheral, timer_driver: TimerDriver<'a>, config: &config::TimerConfig )-> Result<AnalogOut<'a>, AnalogOutError> {

        let ledc_timer_driver = AnalogOut::create_timer_driver(timer, config)?;
        let gpio = gpio_pin.into_any_io_pin().map_err(|_| AnalogOutError::InvalidPeripheral)?;
        
        let pwm_driver =  match peripheral_channel {
            Peripheral::PWMChannel(0) => LedcDriver::new(unsafe {CHANNEL0::new()}, ledc_timer_driver, gpio),
            Peripheral::PWMChannel(1) => LedcDriver::new(unsafe {CHANNEL1::new()}, ledc_timer_driver, gpio),
            Peripheral::PWMChannel(2) => LedcDriver::new(unsafe {CHANNEL2::new()}, ledc_timer_driver, gpio),
            Peripheral::PWMChannel(3) => LedcDriver::new(unsafe {CHANNEL3::new()}, ledc_timer_driver, gpio),
            _ => return Err(AnalogOutError::InvalidPeripheral),
        }.map_err(|_| AnalogOutError::InvalidArg)?;
    
        Ok(AnalogOut{driver: pwm_driver,
            timer_driver: timer_driver, 
            duty: Arc::new(AtomicU32::new(0)), 
            interrupt_update_code: Arc::new(InterruptUpdate::None.get_atomic_code()),
            fixed_change_increasing: Arc::new(AtomicBool::new(false)),
            fixed_change_type: FixedChangeType::None
        })
    }

    /// Creates a new AnalogOut with a default frecuency of 1000Hz and a resolution of 8bits.
    pub fn default(peripheral_channel: Peripheral, timer:Peripheral, gpio_pin: Peripheral, timer_driver: TimerDriver<'a>) -> Result<AnalogOut<'a>, AnalogOutError>{
        AnalogOut::_new(peripheral_channel, timer, gpio_pin, timer_driver, &config::TimerConfig::new())
    }

    /// Creates a new Resolution from a u32 value.
    fn create_resolution(resolution: u32) -> Resolution{
        match resolution {
            0 => Resolution::Bits1,
            1 => Resolution::Bits1,
            2 => Resolution::Bits2,
            3 => Resolution::Bits3,
            4 => Resolution::Bits4,
            5 => Resolution::Bits5,
            6 => Resolution::Bits6,
            7 => Resolution::Bits7,
            8 => Resolution::Bits8,
            9 => Resolution::Bits9,
            10 => Resolution::Bits10,
            11 => Resolution::Bits11,
            12 => Resolution::Bits12,
            13 => Resolution::Bits13,
            _ => Resolution::Bits14,
        }
    }

    /// Creates a new LedcTimerDriver from a Peripheral::PWMTimer
    fn create_timer_driver(timer: Peripheral, config: &config::TimerConfig) -> Result<LedcTimerDriver<'a>, AnalogOutError> {
        let res = match timer {
            Peripheral::PWMTimer(0) => LedcTimerDriver::new(
                unsafe{TIMER0::new()},
                config,
            ),
            Peripheral::PWMTimer(1) => LedcTimerDriver::new(
                unsafe{TIMER1::new()},
                config,
            ),
            Peripheral::PWMTimer(2) => LedcTimerDriver::new(
                unsafe{TIMER2::new()},
                config,
            ),
            Peripheral::PWMTimer(3) => LedcTimerDriver::new(
                unsafe{TIMER3::new()},
                config,
            ),
            Peripheral::None => Err(AnalogOutError::TooManyPWMOutputs)?,
            _ => Err(AnalogOutError::InvalidPeripheral)?
        };

        res.map_err(|error| match error.code(){
            ESP_FAIL => AnalogOutError::InvalidFrequencyOrDuty,
            _ => AnalogOutError::InvalidArg,
        })
    }

    /// Changes the intensity of the signal using the High-Low level ratio
    pub fn set_high_level_output_ratio(&mut self, high_ratio: f32) -> Result<(), AnalogOutError>{
        let duty: u32  = duty_from_high_ratio(self.driver.get_max_duty(), high_ratio);
        self.duty.store(duty, Ordering::SeqCst);
        self.driver.set_duty(duty).map_err(|_| AnalogOutError::ErrorSettingOutput)
    }

    fn start_changing_by_fixed_amount(&mut self, fixed_change_type: FixedChangeType, increase_after_miliseconds: u32, increace_by_ratio: f32, starting_high_ratio: f32)-> Result<(), AnalogOutError>{
        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let duty_ref = self.duty.clone();
        let increase_direction_ref = self.fixed_change_increasing.clone();
        let max_duty = self.driver.get_max_duty();
        
        let starting_duty = duty_from_high_ratio(max_duty, starting_high_ratio);
        duty_ref.store(starting_duty, Ordering::SeqCst);

        let callback = move || {
            let duty_step = duty_from_high_ratio(max_duty, increace_by_ratio).max(1);
            let new_duty = if increase_direction_ref.load(Ordering::Acquire){
                duty_ref.load(Ordering::Acquire) + duty_step
            }else{
                duty_ref.load(Ordering::Acquire) - duty_step
            };
            duty_ref.store(new_duty, Ordering::SeqCst);

            interrupt_update_code_ref.store(InterruptUpdate::ChangeDuty.get_code(), Ordering::SeqCst)
        };
        
        self.timer_driver.interrupt_after(increase_after_miliseconds, callback);
        self.timer_driver.enable().map_err(|err| AnalogOutError::TimerDriverError(err))?;
        self.fixed_change_type = fixed_change_type;
        Ok(())
    }
    /*
    // makes the pin blink for a certain period of time blink_period (micro sec) and in a certain frecuency_micro (micro sec)
    pub fn blink(&mut self, mut amount_of_blinks: u32, time_between_states_micro: u32) -> Result<(), DigitalOutError> {
        amount_of_blinks *= 2;
        if amount_of_blinks == 0 {
            return Ok(())
        }

        let interrupt_update_code_ref = self.interrupt_update_code.clone();
        let callback = move || {
            if amount_of_blinks == 0 {
                interrupt_update_code_ref.store(InterruptUpdate::FinishedBlinking.get_code(), Ordering::SeqCst);
            }else{
                amount_of_blinks -= 1;
                interrupt_update_code_ref.store(InterruptUpdate::KeepBlinking.get_code(), Ordering::SeqCst);
            }
        };
        self.timer_driver.interrupt_after(time_between_states_micro, callback).map_err(|err| DigitalOutError::TimerDriverError(err))?;
        self.timer_driver.enable().map_err(|err| DigitalOutError::TimerDriverError(err))
    }
    
    pub fn update_interrupt(&mut self) -> Result<(), DigitalOutError> {
        let interrupt_update = InterruptUpdate::from_atomic_code(self.interrupt_update_code.clone());
        self.interrupt_update_code.store(InterruptUpdate::None.get_code(), Ordering::SeqCst);
        
        match interrupt_update{
            InterruptUpdate::FinishedBlinking => {self.timer_driver.unsubscribe().map_err(|err| DigitalOutError::TimerDriverError(err))},
            InterruptUpdate::KeepBlinking => {
                self.toggle();
                self.timer_driver.enable().map_err(|err| DigitalOutError::TimerDriverError(err))
                }
                InterruptUpdate::None => Ok(()),
                }
            }
            */

    fn turn_around(&mut self){
        let previouse_direction = self.fixed_change_increasing.load(Ordering::Acquire);
        self.fixed_change_increasing.store(!previouse_direction, Ordering::SeqCst)
    }

    fn change_duty_on_cycle(&mut self)-> Result<(), AnalogOutError>{
        let duty = self.duty.load(Ordering::Acquire);
        self.driver.set_duty(duty).map_err(|_| AnalogOutError::ErrorSettingOutput)?;
        if duty == self.driver.get_max_duty() || duty == 0{
            match self.fixed_change_type {
                FixedChangeType::IncreaseDecrease => self.turn_around(),
                FixedChangeType::DecreaseIncrease => self.turn_around(),
                _ => return self.timer_driver.unsubscribe().map_err(|err| AnalogOutError::TimerDriverError(err)),
            }
        }
        self.timer_driver.enable().map_err(|err| AnalogOutError::TimerDriverError(err))
    }

    pub fn update_interrupt(&mut self) -> Result<(), AnalogOutError> {
        let interrupt_update = InterruptUpdate::from_atomic_code(self.interrupt_update_code.clone());
        self.interrupt_update_code.store(InterruptUpdate::None.get_code(), Ordering::SeqCst);

        if let InterruptUpdate::ChangeDuty = interrupt_update{
            self.change_duty_on_cycle()?
        }
        Ok(())
    }
}

fn duty_from_high_ratio(max_duty: u32, high_ratio: f32) -> u32{
    ((max_duty as f32) * high_ratio) as u32
}