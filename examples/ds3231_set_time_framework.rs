use std::collections::HashMap;

use esp32framework::{serial::I2CMaster, Microcontroller};
use esp_idf_svc::hal::delay::BLOCK;

const DS3231_ADDR: u8 = 0x68;

#[repr(u8)]
enum DS3231_REG_DIR {
    Seconds,
    Minutes,
    Hours,
    Day,
    Date,
    Month,
    Year,
}

enum DAY {
    Sun = 1,
    Mon = 2,
    Tues = 3,
    Wed = 4,
    Thurs = 5,
    Fri = 6,
}

struct DateTime {
    sec: u8,
    min: u8,
    hrs: u8,
    day: u8,
    date: u8,
    month: u8,
    yr: u8,
}

fn bcd_to_decimal(bcd: u8) -> u8 {
    (bcd & 0x0F) + ((bcd >> 4) * 10)
}

fn decimal_to_bcd(decimal: u8) -> u8 {
    ((decimal / 10) << 4) | (decimal % 10)
}

fn write_clock(clock: &mut I2CMaster, time: u8, addr: u8) {
    let bcd_time = decimal_to_bcd(time);
    clock
        .write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
        .unwrap();
}

fn set_time(clock: &mut I2CMaster, start_dt: DateTime) {
    write_clock(clock, start_dt.sec, DS3231_REG_DIR::Seconds as u8);
    write_clock(clock, start_dt.min, DS3231_REG_DIR::Minutes as u8);
    write_clock(clock, start_dt.hrs, DS3231_REG_DIR::Hours as u8);
    write_clock(clock, start_dt.day, DS3231_REG_DIR::Day as u8);
    write_clock(clock, start_dt.date, DS3231_REG_DIR::Date as u8);
    write_clock(clock, start_dt.month, DS3231_REG_DIR::Month as u8);
    write_clock(clock, start_dt.yr, DS3231_REG_DIR::Year as u8);
}

fn parse_read_data(data: [u8; 13] )-> HashMap<String, String>{
    let mut res = HashMap::new();
    let secs = bcd_to_decimal(data[0] & 0x7f);  // 0 1 1 1 1 1 1 1
    let mins = bcd_to_decimal(data[1]);
    let hrs = bcd_to_decimal(data[2] & 0x3f);   // 0 0 1 1 1 1 1 1
    let day_number = bcd_to_decimal(data[4]);
    let month = bcd_to_decimal(data[5]);
    let yr = bcd_to_decimal(data[6]);
    let dow = match bcd_to_decimal(data[3]) {
        1 => "Sunday",
        2 => "Monday",
        3 => "Tuesday",
        4 => "Wednesday",
        5 => "Thursday",
        6 => "Friday",
        7 => "Saturday",
        _ => "",
    };

    res.insert("secs".to_string(), secs.to_string());
    res.insert("min".to_string(), mins.to_string());
    res.insert("hrs".to_string(), hrs.to_string());
    res.insert("dow".to_string(), dow.to_string());
    res.insert("day_number".to_string(), day_number.to_string());
    res.insert("month".to_string(), month.to_string());
    res.insert("year".to_string(), yr.to_string());

    res
}

fn main() {
    esp_idf_svc::sys::link_patches();

    let mut micro = Microcontroller::new();
    let mut ds3231 = micro.set_pins_for_i2c_master(5,6);

    let start_dt = DateTime {
        sec: 0,
        min: 21,
        hrs: 11,
        day: DAY::Sun as u8,
        date: 21,
        month: 7,
        yr: 24,
    };

    set_time(&mut ds3231, start_dt);

    loop {
        let mut data: [u8; 13] = [0_u8; 13];

        // Set reading address in zero to read seconds,minutes,hours,day,day number, month and year
        ds3231.write(DS3231_ADDR, &[0_u8], BLOCK).unwrap();
        ds3231.read(DS3231_ADDR, &mut data, BLOCK).unwrap();

        println!("{:?}", data);

        let parsed_data = parse_read_data(data);

        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02}", parsed_data["dow"], parsed_data["day_number"],
                                                      parsed_data["month"], parsed_data["year"], parsed_data["hrs"], 
                                                      parsed_data["min"], parsed_data["secs"]);

        micro.sleep(1000);
    }
}