use esp32framework::{Microcontroller, gpio::{AnalogIn, InterruptType}};
use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}};
static FLAG: AtomicBool = AtomicBool::new(false);

/// Example using pin GPIO9 as digital in to turn on and off a led
/// on GPIO 3. The signal is configured with a debounce time of 200msec.
fn main(){
    let mut micro = Microcontroller::new();
    let mut button = micro.set_pin_as_digital_in(9);
    let mut led = micro.set_pin_as_digital_out(3);
    button.set_debounce(200 * 1000).unwrap();
    button.trigger_on_interrupt(callback, InterruptType::NegEdge);
    let mut count: i32 = 0;
    
    let mut count :u32 = 0;
    let callback = move || {
        count += 1;
        println!("Press Count {}", count);
        led.toggle();
    };
    micro.wait_for_updates(None)
}