use std::rc::Rc;
use config::Resolution;
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::delay::FreeRtos;
use std::cell::RefCell;

pub type SharableAdcDriver<'a> = Rc<RefCell<Option<AdcDriver<'a, ADC1>>>>;

use crate::gpio::{AnalogInPwm,
    DigitalIn,
    DigitalOut, 
    AnalogIn, 
    AnalogOut};
use crate::serial::I2CMaster;
use crate::utils::timer_driver::TimerDriver;
use crate::microcontroller::peripherals::Peripherals;

pub struct Microcontroller<'a> {
    peripherals: Peripherals,
    timer_driver: Vec<TimerDriver<'a>>,
    adc_driver: SharableAdcDriver<'a>,
}

impl <'a>Microcontroller<'a>{
    pub fn new() -> Self {
        esp_idf_svc::sys::link_patches();
        let mut peripherals = Peripherals::new();
        let timer0 = peripherals.get_timer(0);
        let timer1 = peripherals.get_timer(1);
        
        Microcontroller{
            peripherals: peripherals,
            timer_driver: vec![TimerDriver::new(timer0).unwrap(), TimerDriver::new(timer1).unwrap()],
            adc_driver: Rc::new(RefCell::new(None)),
        }
    }

    /// Creates a DigitalIn on the ESP pin with number 'pin_num' to read digital inputs.
    pub fn set_pin_as_digital_in(&mut self, pin_num: usize)-> DigitalIn<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        DigitalIn::new(self.timer_driver.pop().unwrap(), pin_peripheral).unwrap()
    }
    
    /// Creates a DigitalOut on the ESP pin with number 'pin_num' to writee digital outputs.
    pub fn set_pin_as_digital_out(&mut self, pin_num: usize) -> DigitalOut<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        DigitalOut::new(pin_peripheral, self.timer_driver.pop().unwrap()).unwrap()
    }
    
    /// Starts an adc driver if no other was started before. Bitwidth is always set to 12, since 
    /// the ESP32-C6 only allows that width
    fn start_adc_driver(&mut self) {
        let mut adc_driver = self.adc_driver.borrow_mut();
        if let None = *adc_driver {
            self.peripherals.get_adc();
            let driver = AdcDriver::new(unsafe{ADC1::new()}, &Config::new().resolution(Resolution::Resolution12Bit).calibration(true)).unwrap();
            adc_driver.replace(driver); 
        }
    }

    /// Sets pin as analog input with attenuation set to 2.5dB
    pub fn set_pin_as_analog_in_low_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_2_5}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_2_5}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }
    
    /// Sets pin as analog input with attenuation set to 6dB  
    pub fn set_pin_as_analog_in_medium_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_6}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_6}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }
    
    /// Sets pin as analog input with attenuation set to 11dB  
    pub fn set_pin_as_analog_in_high_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_11}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::DB_11}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }

    /// Sets pin as analog input with attenuation set to 0dB  
    pub fn set_pin_as_analog_in_no_atten(&mut self, pin_num: usize) -> AnalogIn<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogIn::<'a, {attenuation::adc_atten_t_ADC_ATTEN_DB_0}>::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }

    /// 
    pub fn set_pin_as_analog_out(&mut self, pin_num: usize, freq_hz: u32, resolution: u32) -> AnalogOut<'a> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        AnalogOut::<'a>::new(pwm_channel, pwm_timer, pin_peripheral, self.timer_driver.pop().unwrap(), freq_hz, resolution).unwrap()
    } 

    pub fn set_pin_as_default_analog_out(&mut self, pin_num: usize) -> AnalogOut<'a> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        AnalogOut::<'a>::default(pwm_channel, pwm_timer, pin_peripheral, self.timer_driver.pop().unwrap()).unwrap()
    }


    pub fn set_pin_as_analog_in_pwm(&mut self, pin_num: usize, freq_hz: u32) -> AnalogInPwm<'a> {
        
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let timer_driver = self.timer_driver.pop().unwrap();            // TODO: Add a better error. If there is no timers a text should sayy this
        AnalogInPwm::new(timer_driver, pin_peripheral, freq_hz).unwrap()
    }
    
    pub fn set_pin_as_default_analog_in_pwm(&mut self, pin_num: usize) -> AnalogInPwm<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let timer_driver = self.timer_driver.pop().unwrap();
        AnalogInPwm::default(timer_driver, pin_peripheral).unwrap()
    }

    pub fn set_pins_for_i2c(&mut self, sda_pin: usize, scl_pin: usize) {
        let sda_peripheral = self.peripherals.get_digital_pin(sda_pin);
        let scl_peripheral = self.peripherals.get_digital_pin(scl_pin);
        let i2c = self.peripherals.get_i2c();

        I2CMaster::new(sda_peripheral, scl_peripheral, i2c).unwrap();

    }
    
    pub fn update(&mut self, drivers_in: Vec<&mut DigitalIn>, drivers_out: Vec<&mut DigitalOut>) {
        for driver in drivers_in{
            driver.update_interrupt();
        }
        for driver in drivers_out{
            driver.update_interrupt();
        }
        FreeRtos::delay_ms(10_u32);
    }
    
    pub fn sleep(&mut self, miliseconds:u32){
        FreeRtos::delay_ms(miliseconds);
    }
}