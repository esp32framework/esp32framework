use esp32framework::{Microcontroller, gpio::{AnalogIn, InterruptType}};
use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}};
static FLAG: AtomicBool = AtomicBool::new(false);

/// Example using pin GPIO9 as digital in to count the amount of times a button
/// is pressed. The signal is configured with a debounce time of 200msec.
fn main(){
    let mut micro = Microcontroller::new();
    let mut button = micro.set_pin_as_digital_in(9);
    button.set_debounce(200 * 1000).unwrap();
    button.trigger_on_interrupt(callback, InterruptType::NegEdge);
    let mut count: i32 = 0;
    
    loop {
        if FLAG.load(Ordering::Relaxed) {
            FLAG.store(false, Ordering::Relaxed);
            count = count.wrapping_add(1);
            println!("Press Count {}", count);
        }
        micro.update(vec![&mut button], vec![]);
    }
}

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}