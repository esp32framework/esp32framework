//! Example using pin GPIO3 and GPIO4 as analog PWM out in order to control the intensity
//! of the colours Red and Blue of a RGB led.

use esp32framework::Microcontroller;

fn main(){
    let mut micro = Microcontroller::new().unwrap();
    let mut red_analog_out = micro.set_pin_as_default_analog_out(2).unwrap();
    let mut blue_analog_out = micro.set_pin_as_default_analog_out(3).unwrap();
    red_analog_out.start_increasing(100, 0.05, 0.0).unwrap();
    blue_analog_out.start_decreasing_bounce_back(100, 0.05, 0.0, None).unwrap();
    
    micro.wait_for_updates(None)
}
