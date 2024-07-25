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
    Second,
    Minute,
    Hour,
    WeekDay,
    Date,
    Month,
    Year,
}

pub struct DateTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub week_day: u8,
    pub date: u8,
    pub month: u8,
    pub year: u8
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
        self.set(date_time.second, DateTimeComponent::Second)?;
        self.set(date_time.minute, DateTimeComponent::Minute)?;
        self.set(date_time.hour, DateTimeComponent::Hour)?;
        self.set(date_time.week_day, DateTimeComponent::WeekDay)?;
        self.set(date_time.date, DateTimeComponent::Date)?;
        self.set(date_time.month, DateTimeComponent::Month)?;
        self.set(date_time.year, DateTimeComponent::Year)
    }

    pub fn set(&mut self, value: u8, time_component: DateTimeComponent) -> Result<(), I2CError> {
        if !time_component.is_between_boundaries(value) {
            return Err(I2CError::InvalidArg);
        }
        self.write_clock(value, time_component.addr())
    }

    pub fn read_time(&mut self) -> Result<DateTime, I2CError> {
        let mut data: [u8; 7] = [0; 7];

        // The SECONDS_ADDR is written to notify from where we want to start reading
        self.i2c.write(DS3231_ADDR, &[SECONDS_ADDR], BLOCK)?;
        self.i2c.read(DS3231_ADDR, &mut data, BLOCK)?;

        Ok(self.parse_read_data(data))
    }

    pub fn read(&mut self, component: DateTimeComponent) -> Result<u8, I2CError> {
        let mut buffer: [u8; 1] = [0];

        self.i2c.write(DS3231_ADDR, &[component.addr()], BLOCK)?;
        self.i2c.read(DS3231_ADDR, &mut buffer, BLOCK)?;

        let parsed_data = self.parse_component(buffer[0], component);
        Ok(parsed_data)
    }

    fn write_clock(&mut self, time: u8, addr: u8) -> Result<(), I2CError> {
        let bcd_time = self.decimal_to_bcd(time);
        self.i2c.write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
    }

    fn parse_component(&self, read_value: u8, component: DateTimeComponent) -> u8 {
        match component {
            DateTimeComponent::Second => self.bcd_to_decimal(read_value & 0x7f),
            DateTimeComponent::Hour => self.bcd_to_decimal(read_value & 0x7f),
            _ => self.bcd_to_decimal(read_value),
        }
    }

    fn parse_read_data(&self, data: [u8; 7] ) -> DateTime {                     // TODO: Maybe change this name
        // For seconds and hours, a mask is appĺy to ignore unnecessary bits
        let secs = self.bcd_to_decimal(data[0] & 0x7f);  // 0 1 1 1 1 1 1 1
        let mins = self.bcd_to_decimal(data[1]);
        let hrs = self.bcd_to_decimal(data[2] & 0x3f);   // 0 0 1 1 1 1 1 1
        let day_number = self.bcd_to_decimal(data[4]);
        let month = self.bcd_to_decimal(data[5]);
        let yr = self.bcd_to_decimal(data[6]);
        let dow = self.bcd_to_decimal(data[3]); 
        
        DateTime {
            second: secs,
            minute: mins,
            hour: hrs,
            week_day: dow,
            date: day_number,
            month: month,
            year: yr,
        }
        
    }

}



impl DateTimeComponent {

    fn addr(&self) -> u8 {
        match self {
            DateTimeComponent::Second => SECONDS_ADDR,
            DateTimeComponent::Minute => MINUTES_ADDR,
            DateTimeComponent::Hour => HOURS_ADDR,
            DateTimeComponent::WeekDay => DAY_ADDR,
            DateTimeComponent::Date => DATE_ADDR,
            DateTimeComponent::Month => MONTH_ADDR,
            DateTimeComponent::Year => YEAR_ADDR,
        }
    }

    pub fn is_between_boundaries(&self, val: u8) -> bool {
        match self {
            DateTimeComponent::Second => self.check_boundaries(val, MIN_VALUE, MAX_SECS),
            DateTimeComponent::Minute => self.check_boundaries(val, MIN_VALUE, MAX_MINS),
            DateTimeComponent::Hour => true, // TODO: Check if is set on 12 or 24 to see which is the max
            DateTimeComponent::WeekDay => self.check_boundaries(val, MIN_WEEK_DAY, MAX_WEEK_DAY),
            DateTimeComponent::Date => self.check_boundaries(val, MIN_VALUE, MAX_DATE),
            DateTimeComponent::Month => self.check_boundaries(val, MIN_MONTH, MAX_MONTH),
            DateTimeComponent::Year => self.check_boundaries(val, MIN_VALUE, MAX_YEAR),
        }
    }

    fn check_boundaries(&self, val: u8, min_boundarie: u8, max_boundarie: u8) -> bool {
        val >= min_boundarie && val <= max_boundarie
    }
}