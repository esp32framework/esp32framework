//! Example using pin GPIO9 as digital in to count the amount of times a button
//! is pressed and to turn ON and OFF the led connected in GPIO3.
//! The signal is configured with a debounce time of 200msec.

use esp_idf_svc::hal::{gpio::*,peripherals::Peripherals,delay::FreeRtos};
use std::sync::atomic::{AtomicBool, Ordering};

static FLAG: AtomicBool = AtomicBool::new(false);

fn callback(){
    FLAG.store(true, Ordering::Relaxed);
}

fn main(){
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let mut button = PinDriver::input(peripherals.pins.gpio9).unwrap();
    let mut led = PinDriver::output(peripherals.pins.gpio3).unwrap();
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
            
            if led.is_set_high() {
                led.set_low().unwrap();
            }else {
                led.set_high().unwrap();
            }

        }
        button.enable_interrupt().unwrap();
        FreeRtos::delay_ms(200_u32);
    }
}
