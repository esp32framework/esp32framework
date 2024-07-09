use esp32framework::Microcontroller;
use esp32framework::InterruptType;

use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}};

static FLAG: AtomicBool = AtomicBool::new(false);

fn main(){
    let mut micro = Microcontroller::new();

    let mut button = micro.set_pin_as_digital_in(9);
    button.set_debounce(200 * 1000).unwrap();
    button.trigger_on_interrupt(callback, InterruptType::NegEdge);
    
    //button.set_pull(Pull::Down).unwrap();
    let mut count: i32 = 0;
    
    loop {
        if FLAG.load(Ordering::Relaxed) {
            FLAG.store(false, Ordering::Relaxed);

            count = count.wrapping_add(1);
            println!("Press Count {}", count);
        }
        micro.update(vec![&mut button], vec![]);
        micro.sleep(200)
    }
}

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}