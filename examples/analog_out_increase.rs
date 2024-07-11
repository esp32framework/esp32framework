use esp32framework::Microcontroller;

fn main(){
    let mut micro = Microcontroller::new();
    let mut analog_out = micro.set_pin_as_default_analog_out(1);
    let mut analog_in = micro.set_pin_as_analog_in_pwm(5, 1000);
    analog_out.start_increasing_reset(100, 0.05, 0.0, Some(5)).unwrap();

    loop {
        analog_out.update_interrupt().unwrap();
        let level = analog_in.read();
        println!("pin level in {}", level);
        let level_out = analog_out.duty.load(std::sync::atomic::Ordering::Acquire);
        println!("pin level {}", level_out);
        micro.sleep(100);
    }
}