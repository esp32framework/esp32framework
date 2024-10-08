//! Example using pin GPIO5 as analog in to obtain analog messurements of the sound intensity
//! The example takes a smooth reading during the first second, and then takes another 10 readings
//! and compares the sound intensity with the first reading.

use esp32framework::Microcontroller;

fn main() {
    let mut micro = Microcontroller::take();
    let mut sound_in = micro.set_pin_as_analog_in_no_atten(5).unwrap();
    micro.wait_for_updates(Some(2000));
    let first_second = sound_in.smooth_read_during(1000).unwrap() as f32;
    println!("Value of first second #{first_second}");
    micro.wait_for_updates(Some(2000));

    for _ in 0..10 {
        let sound = sound_in.smooth_read_during(1000).unwrap() as f32;
        let louder = (sound - first_second) * 100.0 / first_second;
        println!("sound in is #{louder} % louder than first second");
        micro.wait_for_updates(Some(100));
    }
    println!("\n End of example");
    micro.wait_for_updates(None);
}
