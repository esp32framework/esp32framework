//! Example using pin GPIO5 (sda) and GPIO6 (scl) with i2c to communicate
//! with a ds3231 sensor. Then it will ask the sensor temperature and print it every second.

use esp_idf_svc::hal::{
    delay::{FreeRtos, BLOCK},
    i2c::*,
    peripherals::Peripherals,
    prelude::*,
};

const DS3231_ADDR: u8 = 0x68;

fn twos_complement_to_decimal(value: u8) -> i8 {
    if value & 0x80 != 0 {
        -((!value + 1) as i8)
    } else {
        value as i8
    }
}

fn main() {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut ds3231 = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    loop {
        let mut buffer: [u8; 2] = [0; 2];
        ds3231.write(DS3231_ADDR, &[0x11], BLOCK).unwrap();
        ds3231.read(DS3231_ADDR, &mut buffer, BLOCK).unwrap();

        let temp_lsb = buffer[1] >> 6; // We only need the 2 most significant bits of the LSB

        // Get the MSB part
        let temp_integer = twos_complement_to_decimal(buffer[0]);

        // Transform the LSB part
        let temp_fractional = twos_complement_to_decimal(temp_lsb) as f32 * 0.25;
        let temperature = temp_integer as f32 + temp_fractional;

        println!("The temperature is: {:?} °C", temperature);
        FreeRtos::delay_ms(1000_u32);
    }
}
