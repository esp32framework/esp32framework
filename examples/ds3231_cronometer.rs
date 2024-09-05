//! Example using pin GPIO5 (sda) and GPIO6 (scl) with i2c to communicate
//! with a ds3231 sensor. Then it will ask the sensor for the time and print
//! the raw and parsed data in the screen twice per second.

use std::str;
use esp_idf_svc::hal::{i2c::*,delay::{FreeRtos, BLOCK},peripherals::Peripherals,prelude::*};

const ADDRESS: u8 = 0x68;


fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

fn main() {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut driver = I2cDriver::new(i2c, sda, scl, &config).unwrap();
    let mut buffer = [0u8; 19];

    loop {
        driver.read(ADDRESS, &mut buffer, 50);
        let seconds = bcd_to_decimal(buffer[0]);
        let minutes = bcd_to_decimal(buffer[1]);
        let hours = bcd_to_decimal(buffer[2] & 0x3F);
        let day_of_week = bcd_to_decimal(buffer[3]);
        let day_of_month = bcd_to_decimal(buffer[4]);
        let month = bcd_to_decimal(buffer[5] & 0x1F);
        let year = bcd_to_decimal(buffer[6]) as u32 + 2000;

        println!("Raw data: {:?}",buffer);

        println!("Parsed: {:?}/{:?}/{:?} {:?}:{:?}:{:?}", day_of_week,month,year,hours,minutes,seconds);
        
        FreeRtos::delay_ms(500);
    }

}
