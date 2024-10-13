//! Example demonstrating how to use the UART 1 of the microcontroller.
//! This example Consists of a simple echo that will print everything that 
//! reads in a buffer of size BUFFER_SIZE.
//! The connection should be as follows:
//! TX: Pin 16
//! RX: Pin 17
//! When reading it will show the users inputs. Reading is performed with a 
//! 1-second timeout or until the buffer is full.
//! ## Hardware Compatibility Note
//! For ESP32-C6 users:
//! - An alternative USB connection option is available on this microcontroller.
//! - When using an ESP32-C6, you can utilize the second USB port instead of the previously mentioned pins.
//! - This avoids potential conflicts or limitations associated with the primary UART pins.

use config::StopBits;
use esp_idf_svc::{
    hal::{gpio, peripherals::Peripherals, uart::*, prelude::*, delay::FreeRtos},
    sys::configTICK_RATE_HZ
};

const BUFFER_SIZE: usize = 10;
// To get a 1 second timeout we need to get how many ticks we need according to the constant configTICK_RATE_HZ
const TIMEOUT: u32 = ((configTICK_RATE_HZ as u64) * (1_000_000 as u64) / 1_000_000_u64) as u32;

fn main(){
    esp_idf_svc::hal::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let tx = peripherals.pins.gpio16;
    let rx = peripherals.pins.gpio17;

    println!("Starting UART loopback test");
    let config = config::Config::new().baudrate(Hertz(115_200)).parity_none().stop_bits(StopBits::STOP1);
    let uart = UartDriver::new(
        peripherals.uart1,
        tx,
        rx,
        Option::<gpio::Gpio0>::None,
        Option::<gpio::Gpio1>::None,
        &config,
    ).unwrap();

    loop {
        let mut buffer: [u8;BUFFER_SIZE] = [0;10];
        let _ = uart.read(&mut buffer, TIMEOUT);
        if buffer.iter().any(|&x| x > 0) {
            uart.write(&buffer).unwrap();
        }
        FreeRtos::delay_ms(1000);
    }
}