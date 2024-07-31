use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;
use config::Resolution;
use esp_idf_svc::hal;
use esp_idf_svc::hal::adc::ADC1;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::delay::TICK_RATE_HZ;
use esp_idf_svc::hal::units::Time;
use esp_idf_svc::hal::task::notification::Notification;
use std::cell::RefCell;

pub type SharableAdcDriver<'a> = Rc<RefCell<Option<AdcDriver<'a, ADC1>>>>;

use crate::gpio::AnalogInHighAtten;
use crate::gpio::AnalogInLowAtten;
use crate::gpio::AnalogInMediumAtten;
use crate::gpio::AnalogInNoAtten;
use crate::gpio::{AnalogInPwm,
    DigitalIn,
    DigitalOut, 
    AnalogIn, 
    AnalogOut};
    use crate::utils::timer_driver::TimerDriver;
    use crate::microcontroller::peripherals::Peripherals;

const TICKS_PER_MILLI: f32 = TICK_RATE_HZ as f32 / 1000 as f32;

pub struct Microcontroller<'a> {
    peripherals: Peripherals,
    timer_drivers: Vec<TimerDriver<'a>>,
    adc_driver: SharableAdcDriver<'a>,
    notification: Notification
}

impl <'a>Microcontroller<'a>{
    pub fn new() -> Self {
        esp_idf_svc::sys::link_patches();
        let peripherals = Peripherals::new();
        
        Microcontroller{
            peripherals: peripherals,
            timer_drivers: vec![],
            adc_driver: Rc::new(RefCell::new(None)),
            notification: Notification::new()
        }
    }

    fn get_timer_driver(&mut self)-> TimerDriver<'a>{
        let mut timer_driver = if self.timer_drivers.len() < 2{
            let timer = self.peripherals.get_timer(self.timer_drivers.len());
            TimerDriver::new(timer, self.notification.notifier()).unwrap()
        }else{
            self.timer_drivers.swap_remove(0)
        };

        let timer_driver_copy = timer_driver.create_child_copy().unwrap();
        self.timer_drivers.push(timer_driver);
        return timer_driver_copy;
    }

    /// Creates a DigitalIn on the ESP pin with number 'pin_num' to read digital inputs.
    pub fn set_pin_as_digital_in(&mut self, pin_num: usize)-> DigitalIn<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        DigitalIn::new(self.get_timer_driver(), pin_peripheral).unwrap()
    }
    
    /// Creates a DigitalOut on the ESP pin with number 'pin_num' to writee digital outputs.
    pub fn set_pin_as_digital_out(&mut self, pin_num: usize) -> DigitalOut<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        DigitalOut::new(pin_peripheral, self.get_timer_driver()).unwrap()
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
    pub fn set_pin_as_analog_in_low_atten(&mut self, pin_num: usize) -> AnalogInLowAtten<'a> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogInLowAtten::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }
    
    /// Sets pin as analog input with attenuation set to 6dB  
    pub fn set_pin_as_analog_in_medium_atten(&mut self, pin_num: usize) -> AnalogInMediumAtten<'a> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogInMediumAtten::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }
    
    /// Sets pin as analog input with attenuation set to 11dB  
    pub fn set_pin_as_analog_in_high_atten(&mut self, pin_num: usize) -> AnalogInHighAtten<'a> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogInHighAtten::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }

    /// Sets pin as analog input with attenuation set to 0dB  
    pub fn set_pin_as_analog_in_no_atten(&mut self, pin_num: usize) -> AnalogInNoAtten<'a> {
        self.start_adc_driver();
        let pin_peripheral = self.peripherals.get_analog_pin(pin_num);
        AnalogInNoAtten::new(pin_peripheral, self.adc_driver.clone()).unwrap()
    }

    /// 
    pub fn set_pin_as_analog_out(&mut self, pin_num: usize, freq_hz: u32, resolution: u32) -> AnalogOut<'a> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        AnalogOut::<'a>::new(pwm_channel, pwm_timer, pin_peripheral, self.get_timer_driver(), freq_hz, resolution).unwrap()
    } 

    pub fn set_pin_as_default_analog_out(&mut self, pin_num: usize) -> AnalogOut<'a> {
        let (pwm_channel, pwm_timer) = self.peripherals.get_next_pwm();
        let pin_peripheral = self.peripherals.get_pwm_pin(pin_num);
        AnalogOut::<'a>::default(pwm_channel, pwm_timer, pin_peripheral, self.get_timer_driver()).unwrap()
    }


    pub fn set_pin_as_analog_in_pwm(&mut self, pin_num: usize, freq_hz: u32) -> AnalogInPwm<'a> {
        
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let timer_driver = self.get_timer_driver();            // TODO: Add a better error. If there is no timers a text should sayy this
        AnalogInPwm::new(timer_driver, pin_peripheral, freq_hz).unwrap()
    }
    
    pub fn set_pin_as_default_analog_in_pwm(&mut self, pin_num: usize) -> AnalogInPwm<'a> {
        let pin_peripheral = self.peripherals.get_digital_pin(pin_num);
        let timer_driver = self.get_timer_driver();
        AnalogInPwm::default(timer_driver, pin_peripheral).unwrap()
    }
    
    pub fn update(&mut self, drivers_in: Vec<&mut DigitalIn>, drivers_out: Vec<&mut DigitalOut>) {
        //timer_drivers must be updated before other drivers since this may efect the other drivers updates
        for timer_driver in &mut self.timer_drivers{
            timer_driver.update_interrupts().unwrap();
        }
        for driver in drivers_in{
            driver.update_interrupt().unwrap();
        }
        for driver in drivers_out{
            driver.update_interrupt().unwrap();
        }
    }
    
    pub fn wait_for_updates(&mut self, miliseconds:u32, mut analog_outs: Vec<&mut AnalogOut>){
        let starting_time = Instant::now();
        let mut elapsed = Duration::from_millis(0);
        let sleep_time = Duration::from_millis(miliseconds as u64);

        while elapsed < sleep_time{
            let timeout = ((sleep_time - elapsed).as_millis() as f32 * TICKS_PER_MILLI as f32) as u32;
            if self.notification.wait(timeout).is_some(){
                self.update(vec![], vec![]);
                for analog_out in &mut analog_outs{
                    analog_out.update_interrupt().unwrap();
                    println!("{}", analog_out.duty.load(std::sync::atomic::Ordering::SeqCst));
                }
            }
            elapsed = starting_time.elapsed();
        }   
    }

    pub fn sleep(&mut self, miliseconds:u32){
        FreeRtos::delay_ms(miliseconds)
    }
}