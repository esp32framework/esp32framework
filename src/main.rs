
// use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use esp32framework::sensors::{Alarm1Rate, Alarm2Rate, DateTime, DS3231};
use esp32framework::serial::READER;
use esp32framework::Microcontroller;
// use esp_idf_svc::hal::delay::{FreeRtos, BLOCK};
// use esp_idf_svc::hal::gpio::*;
// use esp_idf_svc::hal::i2c::*;
// use esp_idf_svc::hal::peripherals::Peripherals;
// use esp_idf_svc::hal::prelude::*;
static FLAG: AtomicBool = AtomicBool::new(false);

const DS3231_ADDR: u8 = 0x68;
/*
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

fn write_clock(clock: &mut I2cDriver, time: u8, addr: u8) {
    let bcd_time = decimal_to_bcd(time);
    clock
        .write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
        .unwrap();
}

fn set_time(clock: &mut I2cDriver, secs: u8, min: u8, hrs: u8, day: u8, date: u8, month: u8, year: u8) {
    write_clock(clock, secs, DS3231_REG_DIR::Seconds as u8);
    write_clock(clock, min, DS3231_REG_DIR::Minutes as u8);
    write_clock(clock, hrs, DS3231_REG_DIR::Hours as u8);
    write_clock(clock, day, DS3231_REG_DIR::Day as u8);
    write_clock(clock, date, DS3231_REG_DIR::Date as u8);
    write_clock(clock, month, DS3231_REG_DIR::Month as u8);
    write_clock(clock, year, DS3231_REG_DIR::Year as u8);
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

fn print_data(data: &[u8;19]) {
    let mut v = vec![];
    for i in data {
        v.push(bcd_to_decimal(*i));
    } 
    println!("{:?}", v)
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
        day: DAY::Sun as u8,
        date: 21,
        month: 7,
        yr: 24,
    };
    
    set_time(&mut ds3231, start_dt.sec, start_dt.min, start_dt.hrs, start_dt.day, start_dt.date, start_dt.month, start_dt.yr);

    write_clock(&mut ds3231, 5, 0x07);      // 1 0 0 0 0 1 0 1
    write_clock(&mut ds3231, 22, 0x08);     // 1 0 1 0 0 0 1 0
    write_clock(&mut ds3231, 11, 0x09);     // 1 0 0 1 0 0 0 1 
    write_clock(&mut ds3231, 21, 0x0A);     // 1 0 1 0 0 0 0 1
    // Control 
    write_clock(&mut ds3231, 5, 0x0E);      // 0 0 0 0 0 1 0 1
    // Control/Status
    write_clock(&mut ds3231, 0, 0x0F);    // 0 0 0 0 0 0 0 0
    
    
    // let mut count: i32 = 0;

    loop {
        let mut data: [u8; 19] = [0_u8; 19];

        // Set reading address in zero to read seconds,minutes,hours,day,day number, month and year
        ds3231.write(DS3231_ADDR, &[0_u8], BLOCK).unwrap();
        ds3231.read(DS3231_ADDR, &mut data, BLOCK).unwrap();

        print_data(&data);
        //println!("{:?}", data);

        let parsed_data = parse_read_data(data);
        println!("{:?}", alarm.get_level());
        println!("{}, {}/{}/20{}, {:02}:{:02}:{:02} - status: {:?}", parsed_data["dow"], parsed_data["day_number"],
                                                      parsed_data["month"], parsed_data["year"], parsed_data["hrs"], 
                                                      parsed_data["min"], parsed_data["secs"], bcd_to_decimal(data[15]));

        if parsed_data["secs"] == "10" && parsed_data["min"] == "22" {
            println!("Seteo de la alaram 2");
            write_clock(&mut ds3231, 30, 0x07);      // 1 0 0 0 0 1 0 1
            write_clock(&mut ds3231, 22, 0x08);     // 1 0 1 0 0 0 1 0
            write_clock(&mut ds3231, 11, 0x09);     // 1 0 0 1 0 0 0 1 
            write_clock(&mut ds3231, 21, 0x0A);     // 1 0 1 0 0 0 0 1
        }

        // if FLAG.load(Ordering::Relaxed) {
        //     FLAG.store(false, Ordering::Relaxed);
        //     FreeRtos::delay_ms(200_u32);
        //     if !alarm.is_low(){
        //         continue;
        //     }
        //     count = count.wrapping_add(1);
        //     println!("Press Count {}", count);
        // }
        // alarm.enable_interrupt().unwrap();

        FreeRtos::delay_ms(500_u32);
    }
}
*/
/*
    Con el flag en 0, la salida esta en alto y cuando salta la alarma se pone el flag en 1 y la salida pasa a low
    IMP: La alarma no vuelve a salir si el flag esta en 1
*/

fn main() {
    let mut micro = Microcontroller::new();
    let i2c = micro.set_pins_for_i2c_master(5,6);
    let mut ds3231 = DS3231::new(i2c);
    let sqw = micro.set_pin_as_digital_in(3);

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

        micro.sleep(1000);
    }

}