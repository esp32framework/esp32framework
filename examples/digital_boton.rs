use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use std::{collections::HashMap, sync::atomic::{AtomicBool, Ordering}};

static FLAG: AtomicBool = AtomicBool::new(false);

fn main(){
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();

    let mut button = PinDriver::input(peripherals.pins.gpio9).unwrap();
    
    //button.set_pull(Pull::Down).unwrap();
    button.set_interrupt_type(InterruptType::NegEdge).unwrap();
    let mut count: i32 = 0;
    
    unsafe {
        button.subscribe(callback).unwrap();
    }
    
    button.enable_interrupt().unwrap();
    
    loop {
        if FLAG.load(Ordering::Relaxed) {
            FLAG.store(false, Ordering::Relaxed);
            FreeRtos::delay_ms(200_u32);
            if !button.is_low(){
                continue;
            }

            count = count.wrapping_add(1);
            
            println!("Press Count {}", count);
        }
        button.enable_interrupt().unwrap();
        FreeRtos::delay_ms(200_u32);
    }
}

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}