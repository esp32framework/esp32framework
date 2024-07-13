use esp32framework::Microcontroller;

/// Example using pin GPIO3 and GPIO4 as analog PWM out in order to control the intensity
/// of the colours Red and Blue of a RGB led.
fn main(){
    let mut micro = Microcontroller::new();
    let mut red_analog_out = micro.set_pin_as_default_analog_out(3);
    let mut blue_analog_out = micro.set_pin_as_default_analog_out(4);
    red_analog_out.start_increasing(100, 0.05, 0.0).unwrap();
    blue_analog_out.start_decreasing_bounce_back(100, 0.05, 0.0, None).unwrap();

    loop {
        red_analog_out.update_interrupt().unwrap();
        blue_analog_out.update_interrupt().unwrap();
        micro.sleep(100);
    }
}