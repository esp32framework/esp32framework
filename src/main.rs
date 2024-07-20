use esp32framework::{Microcontroller, gpio::{AnalogIn, InterruptType}};
use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}};
static FLAG: AtomicBool = AtomicBool::new(false);

/// Example using pin GPIO9 as digital in to count the amount of times a button
/// is pressed. The signal is configured with a debounce time of 200msec.
fn main(){
    let mut micro = Microcontroller::new();
    let mut dg1 = micro.set_pin_as_digital_out(3);
    let mut dg2 = micro.set_pin_as_digital_out(4);
    let mut dg3 = micro.set_pin_as_digital_out(5);
    
    loop {
        dg1.toggle().unwrap();
        dg2.toggle().unwrap();
        dg3.toggle().unwrap();
        micro.sleep(100);
        println!("weno")
    }
}

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}