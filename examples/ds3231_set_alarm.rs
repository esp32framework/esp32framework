//! Example using pin GPIO3(sqw), GPIO5 (sda) and GPIO6 (scl) with i2c to set 
//! a date and time with an alarm in a ds3231 sensor. Then it will 
//! ask the sensor for the time and print it with the state of the sqw signal.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use esp_idf_svc::hal::{delay::{FreeRtos, BLOCK},gpio::*,i2c::*,peripherals::Peripherals,prelude::*};
static FLAG: AtomicBool = AtomicBool::new(false);

const DS3231_ADDR: u8 = 0x68;

#[repr(u8)]
enum Ds3231RegDir {
    Day,
    Date,
    Hours,
    Minutes,
    Month,
    Seconds,
    Year,
}

#[allow(dead_code)] // This flag is used so clippy does not bother. This happens because in the example only the Sun is contructed
enum Day {
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

fn write_clock(clock: &mut I2cDriver, time: u8, addr: u8) {
    let bcd_time = decimal_to_bcd(time);
    clock
        .write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
        .unwrap();
}

fn set_time(clock: &mut I2cDriver, date_time: DateTime) {
    write_clock(clock, date_time.sec, Ds3231RegDir::Seconds as u8);
    write_clock(clock, date_time.min, Ds3231RegDir::Minutes as u8);
    write_clock(clock, date_time.hrs, Ds3231RegDir::Hours as u8);
    write_clock(clock, date_time.day, Ds3231RegDir::Day as u8);
    write_clock(clock, date_time.date, Ds3231RegDir::Date as u8);
    write_clock(clock, date_time.month, Ds3231RegDir::Month as u8);
    write_clock(clock, date_time.yr, Ds3231RegDir::Year as u8);
}

fn parse_read_data(data: [u8; 19] )-> HashMap<String, String>{
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

fn callback() {
    FLAG.store(true, Ordering::Relaxed);
}

fn main() {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take().unwrap();
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio5;
    let scl = peripherals.pins.gpio6;
    let mut alarm = PinDriver::input(peripherals.pins.gpio3).unwrap();
    alarm.set_interrupt_type(InterruptType::PosEdge).unwrap();
    unsafe {
        alarm.subscribe(callback).unwrap();
    }
    alarm.enable_interrupt().unwrap();

    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut ds3231 = I2cDriver::new(i2c, sda, scl, &config).unwrap();

    let start_dt = DateTime {
        sec: 0,
        min: 22,
        hrs: 11,
        day: Day::Sun as u8,
        date: 21,
        month: 7,
        yr: 24,
    };
    
    set_time(&mut ds3231, start_dt);

    // Set the alarm at 11:22:05 on date 21.
    // The mask bit for the alarm rate is 0000.
    // This means the alarm will go of when day, hours, minutes, and seconds match
    write_clock(&mut ds3231, 5, 0x07); 
    write_clock(&mut ds3231, 22, 0x08);
    write_clock(&mut ds3231, 11, 0x09);
    write_clock(&mut ds3231, 21, 0x0A);
    // Set the A1IE and INTCN pins to 1 so the Alarm 1 can send a square-wave through SQW pin. 
    write_clock(&mut ds3231, 5, 0x0E);
    // Set the A1F pin to 0 (this is necessary so the square-wave can be sent)
    write_clock(&mut ds3231, 0, 0x0F);
    
    loop {
        let mut data: [u8; 19] = [0_u8; 19];

        // Set reading address in zero to read seconds,minutes,hours,day,day number, month and year
        ds3231.write(DS3231_ADDR, &[0_u8], BLOCK).unwrap();
        ds3231.read(DS3231_ADDR, &mut data, BLOCK).unwrap();

        let parsed_data = parse_read_data(data);
        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02} - SQW: {:?}", parsed_data["dow"], parsed_data["day_number"],
                                                      parsed_data["month"], parsed_data["year"], parsed_data["hrs"], 
                                                      parsed_data["min"], parsed_data["secs"], alarm.get_level());

        FreeRtos::delay_ms(500_u32);
    }
}
