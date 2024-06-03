use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::delay::FreeRtos;
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG: bool = AtomicBool::new(false);

//mod microcontroller;
mod  digital;

fn main(){
    FLAG.store(val, order);
    
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    
    let mut button = PinDriver::input(peripherals.pins.gpio9).unwrap();
    button.set_pull(Pull::Down).unwrap();
    button.set_interrupt_type(InterruptType::PosEdge).unwrap();
    let mut count: i32 = 0;
    
    unsafe {
        button.subscribe(callback).unwrap();
    }
    
    button.enable_interrupt().unwrap();
    
    loop {
        if FLAG.load(Ordering::Relaxed) {
          FLAG.store(false, Ordering::Relaxed);
          count = count.wrapping_add(1);
    
          println!("Press Count {}", count);
          FreeRtos::delay_ms(200_u32);
        }
        button.enable_interrupt().unwrap();
        FreeRtos::delay_ms(20_u32);
    }
    
}
    
fn callback() {
    FLAG.store(true, Ordering::Relaxed);
}

fn suma(a:u8, b:u8)-> u8{
    a+b
}

fn suma3(a:u8, b:u8, c:u8)-> u8{
    a+b+c
}


fn nuestro_main(){
    micro = "micro";
    micro.set_digital_in(9, Pull::Up, callback);

    micro.loop()
}