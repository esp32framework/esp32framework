use std::collections::HashMap;

use esp_idf_svc::{hal::delay::BLOCK, sys::gpio_set_level};

use crate::serial::{I2CError, I2CMaster};

const DS3231_ADDR   : u8 = 0x68;
const SECONDS_ADDR  : u8 = 0x00;
const MINUTES_ADDR  : u8 = 0x01;
const HOURS_ADDR    : u8 = 0x02;
const DAY_ADDR      : u8 = 0x03;
const DATE_ADDR     : u8 = 0x04;
const MONTH_ADDR    : u8 = 0x05;    // Also works for century
const YEAR_ADDR     : u8 = 0x06;


pub struct DS3231<'a> {
    i2c: I2CMaster<'a>
}

impl <'a>DS3231<'a> {
    pub fn new(i2c: I2CMaster<'a>) -> DS3231<'a> {
        DS3231 { i2c }
    }

    fn decimal_to_bcd(&self, decimal: u8) -> u8 {
        ((decimal / 10) << 4) | (decimal % 10)
    }

    fn bcd_to_decimal(&self, bcd: u8) -> u8 {
        (bcd & 0x0F) + ((bcd >> 4) * 10)
    }

    pub fn set_time(&mut self, secs: u8, min: u8, hrs: u8, week_day: u8, date: u8, month: u8, year: u8) -> Result<(), I2CError> { // TODO: Maybe use an struct so parameters are reduced
        self.write_clock(secs, SECONDS_ADDR)?;
        self.write_clock(min, MINUTES_ADDR)?;
        self.write_clock(hrs, HOURS_ADDR)?;
        self.write_clock(week_day, DAY_ADDR)?;
        self.write_clock(date, DATE_ADDR)?;
        self.write_clock(month, MONTH_ADDR)?;
        self.write_clock(year, YEAR_ADDR)
    }

    fn write_clock(&mut self, time: u8, addr: u8) -> Result<(), I2CError> {
        let bcd_time = self.decimal_to_bcd(time);
        self.i2c.write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
    }

    fn parse_read_data(&self, data: [u8; 13] ) -> HashMap<String, String> {
        let mut res = HashMap::new();

        // For seconds and hours, a mask is appĺy to ignore unnecessary bits
        let secs = self.bcd_to_decimal(data[0] & 0x7f);  // 0 1 1 1 1 1 1 1
        let mins = self.bcd_to_decimal(data[1]);
        let hrs = self.bcd_to_decimal(data[2] & 0x3f);   // 0 0 1 1 1 1 1 1
        let day_number = self.bcd_to_decimal(data[4]);
        let month = self.bcd_to_decimal(data[5]);
        let yr = self.bcd_to_decimal(data[6]);
        let dow = self.bcd_to_decimal(data[3]); 
    
        res.insert("seconds".to_string(), secs.to_string());
        res.insert("minutes".to_string(), mins.to_string());
        res.insert("hours".to_string(), hrs.to_string());
        res.insert("day_of_week".to_string(), dow.to_string());
        res.insert("day_number".to_string(), day_number.to_string());
        res.insert("month".to_string(), month.to_string());
        res.insert("year".to_string(), yr.to_string());
    
        res
    }

    pub fn read_time(&mut self) -> Result<HashMap<String, String>, I2CError> {
        let mut data: [u8; 13] = [0_u8; 13];

        self.i2c.write(DS3231_ADDR, &[0_u8], BLOCK)?;
        self.i2c.read(DS3231_ADDR, &mut data, BLOCK)?;

        Ok(self.parse_read_data(data))
    }

}