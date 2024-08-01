use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::uart::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp32framework::{Microcontroller, gpio::{AnalogIn, InterruptType}};


fn main(){
    let mut micro = Microcontroller::new();
    let mut uart = micro.set_pins_for_uart(17,16);

    println!("Starting UART loopback test");

    loop {
        uart.write(b"mensaje\n").unwrap();
        println!("Lo escribi");
        FreeRtos::delay_ms(1000);
    }
}