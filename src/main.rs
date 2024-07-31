use std::sync::mpsc::SendError;

use esp32framework::sensors::HCSR04;
use esp32framework::Microcontroller;
use esp_idf_svc::hal::gpio::PinDriver;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::{Delay, FreeRtos};
use esp_idf_svc::hal::timer::config::Config;
use esp_idf_svc::hal::timer::TimerDriver;
use esp_idf_svc::sys::esp_timer_get_time;


fn main(){

    let mut micro = Microcontroller::new();
    let echo = micro.set_pin_as_digital_in(6);
    let trig = micro.set_pin_as_digital_out(4);
    let mut sensor = HCSR04::new(trig, echo);
    

    let delay = Delay::new_default();

    
    
    loop {
        let distance = sensor.get_distance();
        println!("{:?} cm", distance);
        delay.delay_ms(1000);

    }
}