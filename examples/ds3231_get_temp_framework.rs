//! Example using pin GPIO5 (sda) and GPIO6 (scl) with i2c to communicate
//! with a ds3231 sensor. Then it will ask the sensor temperature and print it every second.

use esp32framework::{Microcontroller, sensors::DS3231};

fn main() {
    let mut micro = Microcontroller::new();
    let i2c = micro.set_pins_for_i2c_master(5,6);
    let mut ds3231 = DS3231::new(i2c);

    loop {
        let temp = ds3231.get_temperature();

        println!("The temperature is: {:?} °C", temp);

        micro.sleep(500);
    }

}
