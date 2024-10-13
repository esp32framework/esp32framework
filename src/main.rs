use config::StopBits;
use esp_idf_svc::{
    hal::{delay::FreeRtos, gpio, peripherals::Peripherals, prelude::*, uart::*},
    sys::configTICK_RATE_HZ,
};

const BUFFER_SIZE: usize = 10;
// To get a 1 second timeout we need to get how many ticks we need according to the constant configTICK_RATE_HZ
const TIMEOUT: u32 = (configTICK_RATE_HZ as u64) as u32;

fn main() {
    esp_idf_svc::hal::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let tx = peripherals.pins.gpio16;
    let rx = peripherals.pins.gpio17;

    println!("Starting UART loopback test");
    let config = config::Config::new()
        .baudrate(Hertz(115_200))
        .parity_none()
        .stop_bits(StopBits::STOP1);
    let uart = UartDriver::new(
        peripherals.uart1,
        tx,
        rx,
        Option::<gpio::Gpio0>::None,
        Option::<gpio::Gpio1>::None,
        &config,
    )
    .unwrap();

    loop {
        let mut buffer: [u8; BUFFER_SIZE] = [0; 10];
        let _ = uart.read(&mut buffer, TIMEOUT);
        if buffer.iter().any(|&x| x > 0) {
            uart.write(&buffer).unwrap();
        }
        FreeRtos::delay_ms(1000);
    }
}
