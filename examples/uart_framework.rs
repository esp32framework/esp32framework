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

use esp32framework::Microcontroller;

const BUFFER_SIZE: usize = 10;

fn main() {
    let mut micro = Microcontroller::take();
    let mut uart = micro.set_pins_for_default_uart(16, 17, 1).unwrap();
    println!("Starting UART loopback test");

    loop {
        let mut buffer: [u8; BUFFER_SIZE] = [0; 10];
        let _ = uart.read_with_timeout(&mut buffer, 1_000_000);
        if buffer.iter().any(|&x| x > 0) {
            uart.write(&buffer).unwrap();
        }
        micro.wait_for_updates(Some(100));
    }
}
