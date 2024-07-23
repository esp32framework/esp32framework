use esp32framework::{sensors::DS3231, Microcontroller};

const SECONDS: u8 = 10;
const MINUTES: u8 = 5;
const HOUR: u8 = 2;
const WEEK_DAY: u8 = 2;
const DATE: u8 = 4;
const MONTH: u8 = 7;
const YEAR: u8 = 24;

fn main() {

    let mut micro = Microcontroller::new();
    let i2c = micro.set_pins_for_i2c_master(5,6);
    let mut ds3231 = DS3231::new(i2c);

    ds3231.set_time(SECONDS, MINUTES, HOUR, WEEK_DAY, DATE, MONTH, YEAR);

    loop {
        // Set reading address in zero to read seconds,minutes,hours,day,day number, month and year
        let parsed_data = ds3231.read_time().unwrap();

        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02}", parsed_data["day_of_week"], parsed_data["day_number"],
                                                      parsed_data["month"], parsed_data["year"], parsed_data["hours"], 
                                                      parsed_data["minutes"], parsed_data["seconds"]);

        micro.sleep(1000);
    }
}