use esp32framework::Microcontroller;

/// Example using pin GPIO3 as analog PWM out in order to control the intensity
/// of the colours Red of a RGB led. The intensity should "bounce" when it reaches
/// the maximum and minimum level.
fn main(){
    let mut micro = Microcontroller::new();
    let mut red_analog_out = micro.set_pin_as_default_analog_out(3);
    red_analog_out.start_increasing_bounce_back(100, 0.05, 0.0, None).unwrap();

    loop {
        micro.update(vec![], vec![]);
        red_analog_out.update_interrupt().unwrap();
        micro.sleep(100);
    }
}