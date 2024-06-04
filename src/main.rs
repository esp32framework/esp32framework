use digital::{Flank, DigitalIn};
//use esp_idf_svc::hal::gpio::*;
//use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use microcontroller::{Microcontroller, InterruptType, Pull};
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG: AtomicBool = AtomicBool::new(false);

mod microcontroller;
mod digital;

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}

fn main(){
    let mut micro = Microcontroller::new();
    let digital_in = micro.set_pin_as_digital_in(9, Flank::Ascending, Pull::Down, InterruptType::PosEdge);
    
    digital_in.trigger_on_flank(callback);
    let mut count: i32 = 0;

    let do_every_loop = move || {
        if FLAG.load(Ordering::Relaxed) {
            FLAG.store(false, Ordering::Relaxed);
            count = count.wrapping_add(1);
      
            println!("Press Count {}", count);
            FreeRtos::delay_ms(200_u32);
          }
    };

    micro.run(do_every_loop);
}