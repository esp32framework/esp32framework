use esp32framework::{sensors::{DateTime, DS3231}, Microcontroller};

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
    
    loop {
        // Set reading address in zero to read seconds,minutes,hours,day,day number, month and year
        let date_time = ds3231.read_time().unwrap();

        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02}", date_time.week_day, date_time.date,
                                                      date_time.month, date_time.year, date_time.hour, 
                                                      date_time.minute, date_time.second);
        
        micro.sleep(1000);
    }
}