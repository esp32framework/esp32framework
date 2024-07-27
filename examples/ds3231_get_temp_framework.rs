use esp32framework::sensors::{Alarm1Rate, DateTime, DS3231};
use esp32framework::serial::READER;
use esp32framework::Microcontroller;

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