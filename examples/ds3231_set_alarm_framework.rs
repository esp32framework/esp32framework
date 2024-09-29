//! Example using pin GPIO3(sqw), GPIO5 (sda) and GPIO6 (scl) with i2c to set 
//! a date and time with an alarm in a ds3231 sensor. Then, every second it will 
//! ask the sensor for the time and print it with the state of the sqw signal.

use esp32framework::{sensors::{Alarm1Rate, DateTime, HourMode, DS3231}, serial::READER, Microcontroller};

fn main() {
    let mut micro = Microcontroller::new().unwrap();
    let i2c = micro.set_pins_for_i2c_master(5,6);
    let mut ds3231 = DS3231::new(i2c, HourMode::TwentyFourHour);
    let sqw = micro.set_pin_as_digital_in(3).unwrap();;

    let date_time = DateTime {
        second: 5,
        minute: 10,
        hour: 20,
        week_day: 4,
        date: 24,
        month: 7,
        year: 24,
    };

    ds3231.set_time(date_time).unwrap();
    ds3231.set_alarm_1(Alarm1Rate::EverySecond).unwrap();

    loop {
        // Set reading address in zero to read seconds, minutes, hours, day, day number, month and year
        let date_time = ds3231.read_and_parse();

        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02} - SQW: {:?}", date_time["dow"], date_time["day_number"],
                                                      date_time["month"], date_time["year"], date_time["hrs"], 
                                                      date_time["min"], date_time["secs"], sqw.get_level());
        
        if sqw.is_low() {
            ds3231.update_alarm_1().unwrap();
        }

        micro.sleep(500);
    }

}
