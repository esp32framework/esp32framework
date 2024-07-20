use esp32framework::Microcontroller;

/// Example using pin GPIO9 as digital in to count the amount of times a button
/// is pressed. The signal is configured with a debounce time of 200msec.
fn main(){
    let mut micro = Microcontroller::new();
    let mut dg1 = micro.set_pin_as_digital_out(3);
    let mut dg2 = micro.set_pin_as_digital_out(4);
    let mut dg3 = micro.set_pin_as_digital_out(5);
    let mut dgin = micro.set_pin_as_digital_in(6);
    
    dg1.blink(5, 1000000).unwrap();
    loop {
        dg2.toggle().unwrap();
        dg3.toggle().unwrap();
        println!("in {:?}", dgin.get_level());
        println!("out {:?}", dg1.get_level());
        micro.update(vec![&mut dgin],vec![&mut dg1, &mut dg2, &mut dg3]);
        micro.sleep(100);
    }
}