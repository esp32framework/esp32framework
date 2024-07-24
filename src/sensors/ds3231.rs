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

const MIN_VALUE     : u8 = 0;
const MAX_SECS      : u8 = 59;
const MAX_MINS      : u8 = 59;
const MIN_WEEK_DAY  : u8 = 1;
const MAX_WEEK_DAY  : u8 = 7;
const MAX_DATE      : u8 = 31;
const MIN_MONTH     : u8 = 1;
const MAX_MONTH     : u8 = 12;
const MAX_YEAR      : u8 = 99;

pub enum DateTimeComponent {
    Second(u8),
    Minute(u8),
    Hour(u8),
    WeekDay(u8),
    Date(u8),
    Month(u8),
    Year(u8),
}

pub struct DateTime { // TODO: Check this. Like this, it is posible to put a DateTimeComponent::Second on the minute field.
    pub second: DateTimeComponent,
    pub minute: DateTimeComponent,
    pub hour: DateTimeComponent,
    pub week_day: DateTimeComponent,
    pub date: DateTimeComponent,
    pub month: DateTimeComponent,
    pub year: DateTimeComponent
} 

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

    pub fn set_time(&mut self, date_time: DateTime) -> Result<(), I2CError> { // TODO: Maybe use an struct so parameters are reduced
        self.set(date_time.second)?;
        self.set(date_time.minute)?;
        self.set(date_time.hour)?;
        self.set(date_time.week_day)?;
        self.set(date_time.date)?;
        self.set(date_time.month)?;
        self.set(date_time.year)
    }

    pub fn set(&mut self, time_component: DateTimeComponent) -> Result<(), I2CError> {
        if !time_component.is_between_boundaries() {
            return Err(I2CError::InvalidArg);
        }
        self.write_clock(time_component.value(), time_component.addr())
    }

    pub fn read_time(&mut self) -> Result<HashMap<String, String>, I2CError> {
        let mut data: [u8; 7] = [0; 7];

        // The SECONDS_ADDR is written to notify from where we want to start reading
        self.i2c.write(DS3231_ADDR, &[SECONDS_ADDR], BLOCK)?;
        self.i2c.read(DS3231_ADDR, &mut data, BLOCK)?;

        Ok(self.parse_read_data(data))
    }

    fn write_clock(&mut self, time: u8, addr: u8) -> Result<(), I2CError> {
        let bcd_time = self.decimal_to_bcd(time);
        self.i2c.write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
    }

    fn parse_read_data(&self, data: [u8; 7] ) -> HashMap<String, String> {
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

}

impl DateTimeComponent {

    pub fn value(&self) -> u8 { // TODO: This is horrible. There has to be another way of getting the value
        match self {
            DateTimeComponent::Second(val) => *val,
            DateTimeComponent::Minute(val) => *val,
            DateTimeComponent::Hour(val) => *val,
            DateTimeComponent::WeekDay(val) => *val,
            DateTimeComponent::Date(val) => *val,
            DateTimeComponent::Month(val) => *val,
            DateTimeComponent::Year(val) => *val,
        }
    }

    fn addr(&self) -> u8 {
        match self {
            DateTimeComponent::Second(_) => SECONDS_ADDR,
            DateTimeComponent::Minute(_) => MINUTES_ADDR,
            DateTimeComponent::Hour(_) => HOURS_ADDR,
            DateTimeComponent::WeekDay(_) => DAY_ADDR,
            DateTimeComponent::Date(_) => DATE_ADDR,
            DateTimeComponent::Month(_) => MONTH_ADDR,
            DateTimeComponent::Year(_) => YEAR_ADDR,
        }
    }

    pub fn is_between_boundaries(&self) -> bool {
        match self {
            DateTimeComponent::Second(val) => self.check_boundaries(*val, MIN_VALUE, MAX_SECS),
            DateTimeComponent::Minute(val) => self.check_boundaries(*val, MIN_VALUE, MAX_MINS),
            DateTimeComponent::Hour(val) => true, // TODO: Check if is set on 12 or 24 to see which is the max
            DateTimeComponent::WeekDay(val) => self.check_boundaries(*val, MIN_WEEK_DAY, MAX_WEEK_DAY),
            DateTimeComponent::Date(val) => self.check_boundaries(*val, MIN_VALUE, MAX_DATE),
            DateTimeComponent::Month(val) => self.check_boundaries(*val, MIN_MONTH, MAX_MONTH),
            DateTimeComponent::Year(val) => self.check_boundaries(*val, MIN_VALUE, MAX_YEAR),
        }
    }

    fn check_boundaries(&self, val: u8, min_boundarie: u8, max_boundarie: u8) -> bool {
        val >= min_boundarie && val <= max_boundarie
    }
}