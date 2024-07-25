use esp32framework::{sensors::{DateTime, DateTimeComponent, DS3231}, serial::READER, Microcontroller};

fn main() {

    let mut micro = Microcontroller::new();
    let i2c = micro.set_pins_for_i2c_master(5,6);
    let mut ds3231 = DS3231::new(i2c);

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
    
    let mut i = 0;
    loop {
        // Set reading address in zero to read seconds, minutes, hours, day, day number, month and year
        let date_time = ds3231.read_and_parse();

        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02}", date_time["dow"], date_time["day_number"],
                                                      date_time["month"], date_time["year"], date_time["hrs"], 
                                                      date_time["min"], date_time["secs"]);

        if i == 4 {
            let second = ds3231.read(DateTimeComponent::Second).unwrap();
            println!("El segundo leido fue: {:?}", second)
        } else if i == 6 {
            let day = ds3231.read(DateTimeComponent::Date).unwrap();
            println!("El dia leido fue: {:?}", day)
        } else if i == 8 {
            let hr = ds3231.read(DateTimeComponent::Hour).unwrap();
            println!("La hora leida fue: {:?}", hr)
        }
        i += 1;
        micro.sleep(1000);
    }
}