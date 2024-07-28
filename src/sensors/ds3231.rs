use std::collections::HashMap;

use esp_idf_svc::hal::delay::BLOCK;

use crate::serial::{I2CError, I2CMaster, READER};

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

const SECONDS_ALARM_1_ADDR    : u8 = 0x07;
const MINUTES_ALARM_1_ADDR    : u8 = 0x08;
const HOURS_ALARM_1_ADDR      : u8 = 0x09;
const DAY_ALARM_1_ADDR        : u8 = 0x0A;

const MINUTES_ALARM_2_ADDR    : u8 = 0x0B;
const HOURS_ALARM_2_ADDR      : u8 = 0x0C;
const DAY_ALARM_2_ADDR        : u8 = 0x0D;

const CONTROL_ADDR          : u8 = 0x0E;
const CONTROL_STATUS_ADDR   : u8 = 0x0F;

const TEMP_ADDR             : u8 = 0x11;

const ALARM_MSB_ON: u8 = 128;

/// Possible alar rates for alarm 1.
/// EverySecond: The alarm activates every second. When using this rate, remember to update the alarm with update_alarm_1(). 
/// Seconds: Receives (second). The alarm activates when seconds match.
/// MinutesAndSeconds: Receives (minutes, seconds). The alarm activates when minutes and seconds match.
/// HoursMinutesAndSeconds: Receives (hours, minutes, seconds). The alarm activates when hour, minutes and seconds match.
/// DateHoursMinutesAndSeconds: Receives (date, hours, minutes, seconds). The alarm activates when date, hour, minutes and seconds match.
/// DayHoursMinutesAndSeconds: Receives (day, hours, minutes, seconds). The alarm activates when day, hour, minutes and seconds match.
pub enum Alarm1Rate {
    EverySecond,
    Seconds(u8),
    MinutesAndSeconds(u8, u8),
    HoursMinutesAndSeconds(u8, u8, u8),
    DateHoursMinutesAndSeconds(u8, u8, u8, u8),
    DayHoursMinutesAndSeconds(u8, u8, u8, u8),
}

/// Possible alar rates for alarm 2.
/// EveryMinute: The alarm activates every minute (00 seconds of every minute).
/// Minutes: Receives (minutes). The alarm activates when minutes match.
/// HoursAndMinutes: Receives (hours, minutes). The alarm activates when hour and minutes match.
/// DateHoursAndMinutes: Receives (date, hours, minutes). The alarm activates when date, hour and minutes.
/// DayHoursAndMinutes: Receives (date, hours, minutes). The alarm activates when date, hour and minutes.
pub enum Alarm2Rate {
    EveryMinute,
    Minutes(u8),
    HoursAndMinutes(u8, u8),
    DateHoursAndMinutes(u8, u8, u8),
    DayHoursAndMinutes(u8, u8, u8),
}

/// Component of DateTime.
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

/// Simple abstraction of the DS3231 that facilitates its handling
pub struct DS3231<'a> {
    i2c: I2CMaster<'a>
}


impl <'a>DS3231<'a> {
    
    /// Simply returns the DS3231 struct
    pub fn new(i2c: I2CMaster<'a>) -> DS3231<'a> {
        DS3231 { i2c }
    }

    /// Receives a decimal and serializies with BCD
    fn decimal_to_bcd(&self, decimal: u8) -> u8 {
        ((decimal / 10) << 4) | (decimal % 10)
    }

    /// Receives a bcd serialized decimal and deserializes it into a decimal
    fn bcd_to_decimal(&self, bcd: u8) -> u8 {
        (bcd & 0x0F) + ((bcd >> 4) * 10)
    }

    /// Receives a DateTime indicating the exact time in which the clock should be
    /// and sets the DS3231 into that time.
    pub fn set_time(&mut self, date_time: DateTime) -> Result<(), I2CError> {
        self.set(date_time.second, DateTimeComponent::Second)?;
        self.set(date_time.minute, DateTimeComponent::Minute)?;
        self.set(date_time.hour, DateTimeComponent::Hour)?;
        self.set(date_time.week_day, DateTimeComponent::WeekDay)?;
        self.set(date_time.date, DateTimeComponent::Date)?;
        self.set(date_time.month, DateTimeComponent::Month)?;
        self.set(date_time.year, DateTimeComponent::Year)
    }

    /// Allows to set just a component (seconds, minutes, hours, etc) of the time to the DS3231.
    pub fn set(&mut self, value: u8, time_component: DateTimeComponent) -> Result<(), I2CError> {
        if !time_component.is_between_boundaries(value) {
            return Err(I2CError::InvalidArg);
        }
        self.write_clock(value, time_component.addr())
    }

    /// Allows to read just a component (seconds, minutes, hours, etc) of the time to the DS3231.
    pub fn read(&mut self, component: DateTimeComponent) -> Result<u8, I2CError> {
        let mut buffer: [u8; 1] = [0];

        self.read_clock(component.addr(), &mut buffer)?;

        let parsed_data = self.parse_component(buffer[0], component);
        Ok(parsed_data)
    }

    /// Reads DS3231 registers starting from 'addr'.
    fn read_clock(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), I2CError> {
        self.i2c.write(DS3231_ADDR, &[addr], BLOCK)?;
        self.i2c.read(DS3231_ADDR, buffer, BLOCK)
    }

    /// Writes DS3231 registers starting from 'addr'.
    fn write_clock(&mut self, time: u8, addr: u8) -> Result<(), I2CError> {
        let bcd_time = self.decimal_to_bcd(time);
        self.i2c.write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
    }

    /// Parses the BCD decimals and uses a bit mask on some registers that have unnecessary bits.
    fn parse_component(&self, read_value: u8, component: DateTimeComponent) -> u8 {
        match component {
            DateTimeComponent::Second => self.bcd_to_decimal(read_value & 0x7f),
            DateTimeComponent::Hour => self.bcd_to_decimal(read_value & 0x7f),
            _ => self.bcd_to_decimal(read_value),
        }
    }

    /// Restarts the whole clock, setting every time component to 0.
    pub fn restart(&mut self) -> Result<(), I2CError> {
        let date_time = DateTime  {
            second: 0,
            minute: 0,
            hour: 0,
            week_day: 0,
            date: 0,
            month: 0,
            year: 0
        };
        self.set_time(date_time)
    }

    /// When reading the whole date and time, the data needs to be parsed.
    /// This is done here and returns a HashMap for the user to have the data
    /// in the easiest way posible. The keys to get the different components are:
    /// Seconds -> secs
    /// Minutes -> min
    /// Hour -> hrs
    /// Day of the week -> dow
    /// Day of the month -> day_number
    /// Month -> month
    /// Year -> year
    fn parse_read_data(&self, data: [u8; 13] )-> HashMap<String, String> {
        let mut res = HashMap::new();
        let secs = self.bcd_to_decimal(data[0] & 0x7f);  // 0 1 1 1 1 1 1 1
        let mins = self.bcd_to_decimal(data[1]);
        let hrs = self.bcd_to_decimal(data[2] & 0x3f);   // 0 0 1 1 1 1 1 1
        let day_number = self.bcd_to_decimal(data[4]);
        let month = self.bcd_to_decimal(data[5]);
        let yr = self.bcd_to_decimal(data[6]);
        let dow = match self.bcd_to_decimal(data[3]) {
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

    /// Reads only 1 register and returns its decimal value
    fn read_last_value(&mut self, addr: u8) -> Result<u8, I2CError> {
        let mut buffer: [u8; 1] = [0];
        self.read_clock(addr, &mut buffer)?;
        let value = self.bcd_to_decimal(buffer[0]);
        Ok(value)
    }

    /// Inside function to set alarm 1. Writes every necessary register.
    fn _set_alarm_1(&mut self, day: u8, hours: u8, minutes:u8, seconds: u8) -> Result<(), I2CError> {
        self.write_clock(seconds, SECONDS_ALARM_1_ADDR)?;
        self.write_clock(minutes, MINUTES_ALARM_1_ADDR)?;
        self.write_clock(hours, HOURS_ALARM_1_ADDR)?;
        self.write_clock(day, DAY_ALARM_1_ADDR)?;

        let mut last_value = self.read_last_value(CONTROL_ADDR)?;
        self.write_clock( last_value | 5 , CONTROL_ADDR)?;

        last_value = self.read_last_value(CONTROL_STATUS_ADDR)?;
        self.write_clock( last_value & 0xFE , CONTROL_STATUS_ADDR)
    }

    /// Inside function to set alarm 2. Writes every necessary register.
    fn _set_alarm_2(&mut self, day: u8, hours: u8, minutes: u8) -> Result<(), I2CError> {
        self.write_clock(minutes, MINUTES_ALARM_2_ADDR)?;
        self.write_clock(hours, HOURS_ALARM_2_ADDR)?;
        self.write_clock(day, DAY_ALARM_2_ADDR)?;
        
        let last_value = self.read_last_value(CONTROL_ADDR)?;
        self.write_clock(last_value | 6, CONTROL_ADDR)?;
        
        let last_status_value = self.read_last_value(CONTROL_STATUS_ADDR)?;
        self.write_clock(last_status_value & 0xFD, CONTROL_STATUS_ADDR)
    }
    
    /// By receiving an Alarm1Rate, it is possible to set different rates on alarm 1.
    pub fn set_alarm_1(&mut self, alarm_rate: Alarm1Rate) -> Result<(), I2CError> {
        match alarm_rate {
            Alarm1Rate::EverySecond => self._set_alarm_1(ALARM_MSB_ON, ALARM_MSB_ON, ALARM_MSB_ON, ALARM_MSB_ON),
            Alarm1Rate::Seconds(secs) => self._set_alarm_1(ALARM_MSB_ON, ALARM_MSB_ON, ALARM_MSB_ON, secs),
            Alarm1Rate::MinutesAndSeconds(mins, secs) => self._set_alarm_1(ALARM_MSB_ON,ALARM_MSB_ON,mins,secs),
            Alarm1Rate::HoursMinutesAndSeconds(hr, mins, secs) => self._set_alarm_1(ALARM_MSB_ON,hr,mins,secs),
            Alarm1Rate::DateHoursMinutesAndSeconds(date, hr, mins, secs) => self._set_alarm_1(date & 0xBF, hr, mins, secs),
            Alarm1Rate::DayHoursMinutesAndSeconds(day, hr, mins, secs) => self._set_alarm_1(day | 0x40, hr, mins, secs),
        }
    }
    
    /// By receiving an Alarm2Rate, it is possible to set different rates on alarm 2.
    pub fn set_alarm_2(&mut self, alarm_rate: Alarm2Rate) -> Result<(), I2CError> {
        match alarm_rate {
            Alarm2Rate::EveryMinute => self._set_alarm_2(ALARM_MSB_ON, ALARM_MSB_ON, ALARM_MSB_ON),
            Alarm2Rate::Minutes(mins) => self._set_alarm_2(ALARM_MSB_ON, ALARM_MSB_ON, mins),
            Alarm2Rate::HoursAndMinutes(hr, mins) => self._set_alarm_2(ALARM_MSB_ON, hr, mins),
            Alarm2Rate::DateHoursAndMinutes(date, hr, mins) => self._set_alarm_2(date & 0xBF, hr, mins),
            Alarm2Rate::DayHoursAndMinutes(day, hr, mins) => self._set_alarm_2(day | 0x40, hr, mins),
        }
    }

    /// Updates the control/status register so the alarm can be activated again. 
    pub fn update_alarm_1(&mut self) -> Result<(), I2CError> {
        // Update the control/status register. To activate the alarm we want to set bit 0 to 0 
        let last_value = self.read_last_value(CONTROL_STATUS_ADDR)?;
        self.write_clock( last_value & 0xFE , CONTROL_STATUS_ADDR)
    }

    /// Updates the control/status register so the alarm can be activated again. 
    pub fn update_alarm_2(&mut self) -> Result<(), I2CError> {
        // Update the control/status register. To activate the alarm we want to set bit 1 to 0 
        let last_value = self.read_last_value(CONTROL_STATUS_ADDR)?;
        self.write_clock( last_value & 0xFD , CONTROL_STATUS_ADDR)
    }

    /// Receives a decimal on two's complement and returns its decimal value.
    fn twos_complement_to_decimal(&self, value: u8) -> f32 {
        if value & 0x80 != 0 {
            -((!value + 1) as f32)
        } else {
            value as f32
        }
    }

    /// Returns the temperature on Celsius.
    pub fn get_temperature(&mut self) -> f32 {
        let mut buffer: [u8; 2] = [0; 2];
        self.i2c.write_read(DS3231_ADDR, &[TEMP_ADDR], &mut buffer,BLOCK).unwrap();

        let temp_integer = self.twos_complement_to_decimal(buffer[0]);
        let temp_fractional = self.twos_complement_to_decimal(buffer[1] >> 6) * 0.25;  // We only need the 2 most significant bits of the register

        temp_integer + temp_fractional
    }


    
}

impl<'a> READER for DS3231<'a> {
    fn read_and_parse<'b>(&'b mut self) -> HashMap<String, String> {
        let mut data: [u8; 13] = [0_u8; 13];
        self.i2c.write(DS3231_ADDR, &[0_u8], BLOCK).unwrap();
        self.i2c.read(DS3231_ADDR, &mut data, BLOCK).unwrap();
        self.parse_read_data(data)
    }
}

impl DateTimeComponent {

    /// Returns the register address of the desired DateTime component.
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

    /// For each component it checks if the value is between possible boundaries set by the bits in the register.
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

    /// Receives a value and the boundaries. Returstrue if the values is between boundaries, else false.
    fn check_boundaries(&self, val: u8, min_boundarie: u8, max_boundarie: u8) -> bool {
        val >= min_boundarie && val <= max_boundarie
    }
}
