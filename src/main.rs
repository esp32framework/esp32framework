
use core::str;

use esp_idf_svc::hal::delay::BLOCK;
use esp_idf_svc::hal::gpio;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::hal::uart::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp32framework::{Microcontroller, gpio::{AnalogIn, InterruptType}, serial::UART};
use esp_idf_svc::sys::UART_NUM_MAX;


fn main(){
    let mut micro = Microcontroller::new();
    let mut uart = micro.set_pins_for_default_uart(10,11, 1);
    let mut buf: [u8; 7] = [0; 7];
    println!("Starting UART loopback test");

    loop {
        let bytes = uart.write(b"mensaje\n").unwrap();
        println!("Lo escribi: {:?} bytes", bytes);
        
        //uart.read_with_timeout(&mut buf,1).unwrap();
        uart.read_with_timeout(&mut buf, 1).unwrap();
        
        println!("Lo lei: {:?} bytes", buf);
        println!("Lo lei en ascii: {:?} bytes", str::from_utf8(&buf));
        FreeRtos::delay_ms(1000);
    }
}








// use esp_idf_svc::hal::delay::BLOCK;
// use esp_idf_svc::hal::gpio;
// use esp_idf_svc::hal::peripherals::Peripherals;
// use esp_idf_svc::hal::prelude::*;
// use esp_idf_svc::hal::uart::*;
// use esp_idf_svc::hal::delay::FreeRtos;


// fn main(){
//     esp_idf_svc::hal::sys::link_patches();

//     let peripherals = Peripherals::take().unwrap();
//     let tx = peripherals.pins.gpio16;
//     let rx = peripherals.pins.gpio17;

//     println!("Starting UART loopback test");
//     let config = config::Config::new().baudrate(Hertz(115_200));
//     let uart = UartDriver::new(
//         peripherals.uart2,
//         tx,
//         rx,
//         Option::<gpio::Gpio0>::None,
//         Option::<gpio::Gpio1>::None,
//         &config,
//     ).unwrap();

//     loop {
//         uart.write(b"mensaje\n").unwrap();
//         println!("Lo escribi");
//         FreeRtos::delay_ms(1000);
//     }
// }