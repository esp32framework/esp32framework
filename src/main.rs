//! Example using pin GPIO5 (sda) and GPIO6 (scl) with i2c to set 
//! a date and time in a ds3231 sensor. Then it will ask the sensor
//! for the time and print it in the screen.

use esp32framework::{sensors::{DateTime, HourMode, DS3231}, serial::READER, Microcontroller};

fn main() {

    let mut micro = Microcontroller::new();
    let i2c = micro.set_pins_for_i2c_master(5,6);
    let mut ds3231 = DS3231::new(i2c, HourMode::TwelveHourPM);

    let date_time = DateTime {
        second: 50,
        minute: 59,
        hour: 12,
        week_day: 4,
        date: 24,
        month: 7,
        year: 24,
    };

    ds3231.set_time(date_time).unwrap();
    
    loop {
        // Set reading address in zero to read seconds, minutes, hours, day, day number, month and year
        let date_time = ds3231.read_and_parse();

        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02}", date_time["dow"], date_time["day_number"],
                                                      date_time["month"], date_time["year"], date_time["hrs"], 
                                                      date_time["min"], date_time["secs"]);

        let meridiem = ds3231.meridiem().unwrap();

        println!("{:?}", meridiem);

        micro.sleep(1000);
    }
}
