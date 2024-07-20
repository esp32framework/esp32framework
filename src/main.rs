//! I2C test with SSD1306
//!
//! Folowing pins are used:
//! SDA     GPIO5
//! SCL     GPIO6
//!
//! Depending on your target and the board you are using you have to change the pins.
//!
//! For this example you need to hook up an SSD1306 I2C display.
//! The display will flash black and white.

use esp_idf_svc::hal::delay::{FreeRtos, BLOCK};
use esp_idf_svc::hal::i2c::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

use std::str;

// mod i2c;


const ADDRESS: u8 = 0x68;


fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

// fn read_values -> dict{
//     driver.read();
//     parse...
//     return [hour: '',minu:]
// }


fn main() {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut driver = I2cDriver::new(i2c, sda, scl, &config).unwrap();
    let mut buffer = [0u8; 19];
    let mut data: [u8; 8] = [0; 8];

    // ejecutar_fuera_de_rango(hour,0 , 10, fn prender_luz)

    data[0] = (0 / 10 * 16) + (0 % 10);
    data[1] = (10 / 10 * 16) + (10 % 10);
    data[2] = (1 / 10 * 16) + (1 % 10);
    data[3] = 5;
    data[4] = (19 / 10 * 16) + (19 % 10);
    data[5] = (7 / 10 * 16) + (7 % 10);
    data[6] = ((2024 / 10 * 16) + (2024 % 10)) as u8;

    // Control register (not setting CH bit to enable oscillator)
    data[7] = 0x00;

    // Escribir los datos en el DS3231
    driver.write(ADDRESS, &data, 10).unwrap();


    loop {
        driver.read(ADDRESS, &mut buffer, 50);
        //let read_val = str::from_utf8(&buffer).unwrap();
        let seconds = bcd_to_decimal(buffer[0]);
        let minutes = bcd_to_decimal(buffer[1]);
        let hours = bcd_to_decimal(buffer[2] & 0x3F); // Ignorar el bit de formato 12/24 horas
        let day_of_week = bcd_to_decimal(buffer[3]);
        let day_of_month = bcd_to_decimal(buffer[4]);
        let month = bcd_to_decimal(buffer[5] & 0x1F); // Ignorar el bit del siglo
        let year = bcd_to_decimal(buffer[6]) as u32 + 2000; // Asumir siglo 21

        println!("La raw es: {:?} y pasado a str es",buffer);

        println!("{:?}/{:?}/{:?} {:?}:{:?}:{:?}", day_of_week,month,year,hours,minutes,seconds);
        
        FreeRtos::delay_ms(500);
    }

}