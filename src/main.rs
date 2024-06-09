use digital_in::{DigitalIn, InterruptUpdate};
use error_text_parser::map_enable_disable_errors;
//use esp_idf_svc::hal::gpio::*;
//use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use microcontroller::Microcontroller;
pub use esp_idf_svc::hal::gpio::{InterruptType, Pull};
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG: AtomicBool = AtomicBool::new(false);

mod microcontroller;
mod digital_in;
mod digital_out;
mod error_text_parser;

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}


//pull up default = 1
//pull|interrupt|funciona
//up  |  neg    | si
//up  |  pos    | si

//down|  neg    | al reves
//down|  pos    | al reves

// up  | neg      | 0
// dow | neg     | 1

fn main(){
    let mut micro = Microcontroller::new();
    let mut digital_in = micro.set_pin_as_digital_in(9, InterruptType::HighLevel);
    //let mut digital_out = micro.set_pin_as_digital_out(10);
    //digital_in.set_pull(Pull::Down).unwrap();
    //digital_in.set_debounce(2000);
    digital_in.trigger_on_flank(callback).unwrap();
    let mut count: i32 = 0;
    
    let mut i = 0;
    
    loop {
        if FLAG.load(Ordering::Relaxed) {
            FLAG.store(false, Ordering::Relaxed);
            count = count.wrapping_add(1);
            
            println!("Press Count {}", count);
            FreeRtos::delay_ms(200_u32);
        }
        
        //digital_out.toggle().unwrap();
        //println!("Out: {:?}", digital_out.get_level());
        FreeRtos::delay_ms(200_u32);
        println!("In: {:?}", digital_in.get_level());
        micro.update(vec![&mut digital_in]);
        
    }
}