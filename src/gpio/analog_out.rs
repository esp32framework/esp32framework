// use esp_idf_svc::hal::gpio::Gpio0;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::sys::ESP_FAIL;
use crate::microcontroller::peripherals::Peripheral;

pub struct AnalogOut<'a> {
    driver: LedcDriver<'a>
}

#[derive(Debug)]
pub enum AnalogOutError{
    TooManyPWMOutputs,
    InvalidArg,
    InvalidPeripheral,
    InvalidFrequencyOrDuty,
    ErrorSettingOutput,
}

impl <'a>AnalogOut<'a> {
    //TODO: Dejar elegir al usuario low y high resolution, segun que timer
    
    pub fn new(peripheral_channel: Peripheral, timer:Peripheral, gpio_pin: Peripheral, freq_hz: u32, resolution: u32) -> Result<AnalogOut<'a>, AnalogOutError> {
        let resolution = AnalogOut::create_resolution(resolution);
        let config = &config::TimerConfig::new().frequency(freq_hz.Hz().into()).resolution(resolution);
        AnalogOut::_new(peripheral_channel, timer, gpio_pin, config)
    }
    
    pub fn _new(peripheral_channel: Peripheral, timer:Peripheral, gpio_pin: Peripheral, config: &config::TimerConfig )-> Result<AnalogOut<'a>, AnalogOutError> {

        let timer_driver = AnalogOut::create_timer_driver(timer, config)?;
        let gpio = gpio_pin.into_any_io_pin().map_err(|_| AnalogOutError::InvalidPeripheral)?;
        
        let pwm_driver =  match peripheral_channel {
            Peripheral::PWMChannel(0) => LedcDriver::new(unsafe {CHANNEL0::new()}, timer_driver, gpio),
            Peripheral::PWMChannel(1) => LedcDriver::new(unsafe {CHANNEL1::new()}, timer_driver, gpio),
            Peripheral::PWMChannel(2) => LedcDriver::new(unsafe {CHANNEL2::new()}, timer_driver, gpio),
            Peripheral::PWMChannel(3) => LedcDriver::new(unsafe {CHANNEL3::new()}, timer_driver, gpio),
            _ => return Err(AnalogOutError::InvalidPeripheral),
        }.map_err(|_| AnalogOutError::InvalidArg)?;

        Ok(AnalogOut{driver: pwm_driver})
    }

    pub fn default(peripheral_channel: Peripheral, timer:Peripheral, gpio_pin: Peripheral) -> Result<AnalogOut<'a>, AnalogOutError>{
        AnalogOut::_new(peripheral_channel, timer, gpio_pin, &config::TimerConfig::new())
    }

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

    pub fn set_high_level_output_ratio(&mut self, high_ratio: f32) -> Result<(), AnalogOutError>{
        let duty: u32  = ((self.driver.get_max_duty() as f32) * high_ratio) as u32;
        self.driver.set_duty(duty).map_err(|_| AnalogOutError::ErrorSettingOutput)
    }

    fn set_frequency(self){

    }

    fn set_resolution(self){
        
    }
}