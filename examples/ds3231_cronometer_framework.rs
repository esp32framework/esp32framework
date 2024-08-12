use esp32framework::{Microcontroller, gpio::{AnalogIn, InterruptType}};
use std::{str, collections::HashMap, sync::atomic::{AtomicBool, Ordering}};

const ADDRESS: u8 = 0x68;


fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

fn main() {
    let mut micro = Microcontroller::new();
    let mut driver = micro.set_pins_for_i2c_master(5,6);
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
        
        micro.sleep(500)
    }
}