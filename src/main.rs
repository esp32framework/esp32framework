use esp32framework::Microcontroller;

fn main(){

    let mut micro = Microcontroller::new();
    let mut sound_in = micro.set_pin_as_analog_in_no_atten(5);
    micro.wait_for_updates(Some(2000));
    
    for _ in 0..100{
        let sound = sound_in.smooth_read(10).unwrap();
        println!("in: #{sound}");
        micro.wait_for_updates(Some(100));
    }
    println!("\n End of example");
    micro.wait_for_updates(None);
}
