//! Example using pin GPIO3 as analog PWM out in order to control the intensity
//! of the colours Red of a RGB led. The intensity should "bounce" when it reaches
//! the maximum and minimum level.

use esp32framework::Microcontroller;

fn main(){
    let mut micro = Microcontroller::new();
    let mut red_analog_out = micro.set_pin_as_default_analog_out(3);
    red_analog_out.start_increasing_bounce_back(100, 0.05, 0.0, None).unwrap();

    micro.wait_for_updates(None)
}