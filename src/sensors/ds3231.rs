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

const MIN_VALUE         : u8 = 0;
const MAX_SECS          : u8 = 59;
const MAX_MINS          : u8 = 59;
const MIN_WEEK_DAY      : u8 = 1;
const MAX_WEEK_DAY      : u8 = 7;
const MAX_DATE          : u8 = 31;
const MIN_MONTH         : u8 = 1;
const MAX_MONTH         : u8 = 12;
const MAX_YEAR          : u8 = 99;
const MAX_12_HOUR_MODE  : u8 = 12;
const MAX_24_HOUR_MODE  : u8 = 24;

const SECONDS_ALARM_1_ADDR    : u8 = 0x07;
const MINUTES_ALARM_1_ADDR    : u8 = 0x08;
const HOURS_ALARM_1_ADDR      : u8 = 0x09;
const DAY_ALARM_1_ADDR        : u8 = 0x0A;

const MINUTES_ALARM_2_ADDR    : u8 = 0x0B;
const HOURS_ALARM_2_ADDR      : u8 = 0x0C;
const DAY_ALARM_2_ADDR        : u8 = 0x0D;

const READ_BITMASK_24_HOUR_MODE  : u8 = 0x3f;
const READ_BITMASK_12_HOUR_MODE  : u8 = 0x1f;
const READ_BITMASK_SECONDS       : u8 = 0x7f;

const WRITE_BITMASK_24_HOUR_MODE        : u8 = 0x00;
const WRITE_BITMASK_12_AM_HOUR_MODE     : u8 = 0x40;
const WRITE_BITMASK_12_PM_HOUR_MODE     : u8 = 0x60;

const CONTROL_ADDR          : u8 = 0x0E;
const CONTROL_STATUS_ADDR   : u8 = 0x0F;

const TEMP_ADDR             : u8 = 0x11;

const ALARM_MSB_ON: u8 = 128;

const MERIDIEM_BITMASK : u8 = 0x20;

/// Possible alar rates for alarm 1.
/// - `EverySecond`: The alarm activates every second. When using this rate, remember to update the alarm with update_alarm_1(). 
/// - `Seconds`: Receives (second). The alarm activates when seconds match.
/// - `MinutesAndSeconds`: Receives (minutes, seconds). The alarm activates when minutes and seconds match.
/// - `HoursMinutesAndSeconds`: Receives (hours, minutes, seconds). The alarm activates when hour, minutes and seconds match.
/// - `DateHoursMinutesAndSeconds`: Receives (date, hours, minutes, seconds). The alarm activates when date, hour, minutes and seconds match.
/// - `DayHoursMinutesAndSeconds`: Receives (day, hours, minutes, seconds). The alarm activates when day, hour, minutes and seconds match.
pub enum Alarm1Rate {
    EverySecond,
    Seconds(u8),
    MinutesAndSeconds(u8, u8),
    HoursMinutesAndSeconds(u8, u8, u8),
    DateHoursMinutesAndSeconds(u8, u8, u8, u8),
    DayHoursMinutesAndSeconds(u8, u8, u8, u8),
}

/// Possible alar rates for alarm 2.
/// - `EveryMinute`: The alarm activates every minute (00 seconds of every minute).
/// - `Minutes`: Receives (minutes). The alarm activates when minutes match.
/// - `HoursAndMinutes`: Receives (hours, minutes). The alarm activates when hour and minutes match.
/// - `DateHoursAndMinutes`: Receives (date, hours, minutes). The alarm activates when date, hour and minutes.
/// - `DayHoursAndMinutes`: Receives (date, hours, minutes). The alarm activates when date, hour and minutes.
pub enum Alarm2Rate {
    EveryMinute,
    Minutes(u8),
    HoursAndMinutes(u8, u8),
    DateHoursAndMinutes(u8, u8, u8),
    DayHoursAndMinutes(u8, u8, u8),
}

/// Component of DateTime.
#[derive(PartialEq)]
pub enum DateTimeComponent {
    Second,
    Minute,
    Hour,
    WeekDay,
    Date,
    Month,
    Year,
}

/// Represents a date and time
/// - `second`: The second component of the time (0-59).
/// - `minute`: The minute component of the time (0-59).
/// - `hour`: The hour component of the time (0-23).
/// - `week_day`: The day of the week (1-7).
/// - `date`: The day of the month (1-31).
/// - `month`: The month (1-12).
/// - `year`: The year component (0-99, representing 2000-2099).
pub struct DateTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub week_day: u8,
    pub date: u8,
    pub month: u8,
    pub year: u8,
}

/// Enums the different possible hour modes for the DS3231:
/// - `TwentyFourHour`: Hours will go from 0 to 24
/// - `TwelveHourAM`: Hours will go from 0 to 12. It will satart with the AM mode
/// - `TwelveHourPM`: Hours will go from 0 to 12. It will satart with the PM mode
#[derive(Clone, Copy, Debug)]
pub enum HourMode {
    TwentyFourHour,
    TwelveHourAM,
    TwelveHourPM,
}

/// Enums the meridiems used on the DS3231
#[derive(PartialEq, Debug)]
pub enum Meridiem {
    AM,
    PM,
}

/// Simple abstraction of the DS3231 that facilitates its handling
pub struct DS3231<'a> {
    i2c: I2CMaster<'a>,
    mode: HourMode
}


impl <'a>DS3231<'a> {
    
    /// Creates a new `DS3231` instance with 24-hour mode.
    ///
    /// # Arguments
    ///
    /// - `i2c`: The I2CMaster interface to communicate with the DS3231.
    ///
    /// # Returns
    ///
    /// A new `DS3231` instance.
    pub fn new(i2c: I2CMaster<'a>) -> DS3231<'a> {
        DS3231 { i2c, mode: HourMode::TwentyFourHour }
    }

    /// Creates a new `DS3231` instance with the desired hour mode.
    ///
    /// # Arguments
    ///
    /// - `i2c`: The I2CMaster interface to communicate with the DS3231.
    /// - `mode`: The desired hour mode for the clock.
    ///
    /// # Returns
    ///
    /// A new `DS3231` instance.
    pub fn new_with_hour_mode(i2c: I2CMaster<'a>, mode: HourMode) -> DS3231<'a> {
        DS3231 { i2c, mode }
    } 

    /// Converts a decimal number to its Binary-Coded Decimal (BCD) representation.
    ///
    /// # Arguments
    ///
    /// - `decimal`: The decimal value to be converted.
    ///
    /// # Returns
    ///
    /// The BCD representation of the decimal value.
    fn decimal_to_bcd(&self, decimal: u8) -> u8 {
        ((decimal / 10) << 4) | (decimal % 10)
    }

    /// Gets the meridiem set on the DS3231.
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the Meridiem type or an `I2CError` if
    /// reading the DS3231 fails
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn meridiem(&mut self) -> Result<Meridiem, I2CError> {
        let mut buffer = [0_u8; 1];
        self.read_clock(DateTimeComponent::Hour.addr(), &mut buffer)?;

        let mut read_byte = buffer[0];
        read_byte &= MERIDIEM_BITMASK;

        if read_byte == MERIDIEM_BITMASK {
            return Ok(Meridiem::PM);
        }
        Ok(Meridiem::AM)
    }

    /// Converts a Binary-Coded Decimal (BCD) value to its decimal representation.
    ///
    /// # Arguments
    ///
    /// - `bcd`: The BCD value to be converted.
    ///
    /// # Returns
    ///
    /// The decimal representation of the BCD value.
    fn bcd_to_decimal(&self, bcd: u8) -> u8 {
        (bcd & 0x0F) + ((bcd >> 4) * 10)
    }

    /// Receives a DateTime indicating the exact time in which the clock should be
    /// and sets the DS3231 into that time.
    /// 
    /// # Arguments
    ///
    /// - `date_time`: The DateTime struct representing the desired time.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the time was successfully set, otherwise an `I2CError`
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If any of the value of the DateTime is not between acceptable boundaries
    pub fn set_time(&mut self, date_time: DateTime) -> Result<(), I2CError> {
        self.set(date_time.second, DateTimeComponent::Second)?;
        self.set(date_time.minute, DateTimeComponent::Minute)?;
        self.set(date_time.week_day, DateTimeComponent::WeekDay)?;
        self.set(date_time.date, DateTimeComponent::Date)?;
        self.set(date_time.month, DateTimeComponent::Month)?;
        self.set(date_time.year, DateTimeComponent::Year)?;
        self.set(date_time.hour, DateTimeComponent::Hour)
    }


    /// Retrieves a date_time with the current date and time. 
    /// 
    /// # Returns
    ///
    /// A `DateTime` struct representing the current date and time.
    pub fn get_date_time(&mut self) -> DateTime {
        let date_time = self.read_and_parse();
        DateTime {
            second: date_time["secs"].parse().unwrap(),
            minute: date_time["min"].parse().unwrap(),
            hour: date_time["hrs"].parse().unwrap(),
            week_day: self.read(DateTimeComponent::WeekDay).unwrap(),
            date: date_time["day_number"].parse().unwrap(),
            month: date_time["month"].parse().unwrap(),
            year: date_time["year"].parse().unwrap(),
        }
    }

    /// Allows to set just a component (seconds, minutes, hours, etc) of the time to the DS3231.
    /// 
    /// # Arguments
    ///
    /// - `value`: The value to set the component to.
    /// - `time_component`: The specific component of time (seconds, minutes, hours, etc.) to set.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the component was successfully set, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If any of the value of the DateTime is not between acceptable boundaries
    pub fn set(&mut self, value: u8, time_component: DateTimeComponent) -> Result<(), I2CError> {
        if !time_component.is_between_boundaries(value, self.mode) {
            return Err(I2CError::InvalidArg);
        }
        self.write_clock(value, time_component.addr())
    }

    /// Allows to read just a component (seconds, minutes, hours, etc) of the time to the DS3231.
    /// 
    /// # Arguments
    ///
    /// - `component`: The specific component of time (seconds, minutes, hours, etc.) to read.
    ///
    /// # Returns
    ///
    /// The value of the component read, or an `I2CError` if the read operation failed.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn read(&mut self, component: DateTimeComponent) -> Result<u8, I2CError> {
        let mut buffer: [u8; 1] = [0];

        self.read_clock(component.addr(), &mut buffer)?;

        let parsed_data = self.parse_component(buffer[0], component);
        Ok(parsed_data)
    }

    /// Reads DS3231 registers starting from 'addr'.
    /// 
    /// # Arguments
    ///
    /// - `addr`: The starting register address to read from.
    /// - `buffer`: A mutable buffer to store the data read from the registers.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the read operation was successful, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    fn read_clock(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), I2CError> {
        self.i2c.write(DS3231_ADDR, &[addr], BLOCK)?;
        self.i2c.read(DS3231_ADDR, buffer, BLOCK)
    }

    /// Writes DS3231 registers starting from 'addr'.
    /// 
    /// # Arguments
    ///
    /// - `time`: The time value to write, in decimal format.
    /// - `addr`: The starting register address to write to.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the write operation was successful, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    fn write_clock(&mut self, time: u8, addr: u8) -> Result<(), I2CError> {
        let mut bcd_time = self.decimal_to_bcd(time);

        // If the hour is being written, then a mask is used to keep the hour-mode bits correct
        if addr == DateTimeComponent::Hour.addr() {
            let bitmask = self.mode.get_write_bitmask();
            bcd_time |= bitmask;
        }
        self.i2c.write(DS3231_ADDR, &[addr, bcd_time], BLOCK)
    }

    /// Parses the BCD decimals and uses a bit mask on some registers that have unnecessary bits.
    /// 
    /// # Arguments
    ///
    /// - `read_value`: The BCD-encoded value read from the DS3231.
    /// - `component`: The `DateTimeComponent` representing the type of time component (e.g., second, hour).
    ///
    /// # Returns
    ///
    /// The decimal representation of the time component, with unnecessary bits masked out if applicable.
    fn parse_component(&self, read_value: u8, component: DateTimeComponent) -> u8 {
        match component {
            DateTimeComponent::Second => self.bcd_to_decimal(read_value & READ_BITMASK_SECONDS),
            DateTimeComponent::Hour => self.bcd_to_decimal(read_value & self.mode.get_read_bitmask()),
            _ => self.bcd_to_decimal(read_value),
        }
    }

    /// Restarts the whole clock, setting every time component to 0.
    /// 
    /// # Returns
    ///
    /// `Ok(())` if the clock was successfully restarted, otherwise an `I2CError`.
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
    /// - Seconds -> secs
    /// - Minutes -> min
    /// - Hour -> hrs
    /// - Day of the week -> dow
    /// - Day of the month -> day_number
    /// - Month -> month
    /// - Year -> year
    /// 
    /// # Arguments
    ///
    /// - `data`: An array of 13 bytes representing the raw data read from the DS3231.
    ///
    /// # Returns
    ///
    /// A `HashMap` where the keys are string representations of the time components (e.g., "secs", "min") and the values are their corresponding decimal values.
    fn parse_read_data(&self, data: [u8; 13] )-> HashMap<String, String> {
        let mut res = HashMap::new();
        let secs = self.bcd_to_decimal(data[0] & READ_BITMASK_SECONDS);
        let mins = self.bcd_to_decimal(data[1]);
        let hrs = self.bcd_to_decimal(data[2] & self.mode.get_read_bitmask());
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
    /// 
    /// # Arguments
    ///
    /// - `addr`: The register address to read from.
    ///
    /// # Returns
    ///
    /// The decimal value of the register, or an `I2CError` if the read operation failed.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    fn read_last_value(&mut self, addr: u8) -> Result<u8, I2CError> {
        let mut buffer: [u8; 1] = [0];
        self.read_clock(addr, &mut buffer)?;
        let value = self.bcd_to_decimal(buffer[0]);
        Ok(value)
    }

    /// Inside function to set alarm 1. Writes every necessary register.
    /// 
    /// # Arguments
    ///
    /// - `day`: The day of the month for the alarm.
    /// - `hours`: The hour for the alarm.
    /// - `minutes`: The minute for the alarm.
    /// - `seconds`: The second for the alarm.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the alarm was successfully set, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
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
    /// 
    /// # Arguments
    ///
    /// - `day`: The day of the month for the alarm.
    /// - `hours`: The hour for the alarm.
    /// - `minutes`: The minute for the alarm.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the alarm was successfully set, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
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
    /// 
    /// # Arguments
    ///
    /// - `alarm_rate`: The rate at which alarm 1 should trigger. This can be one of several predefined rates in Alarm1Rate.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the alarm rate was successfully set, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
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
    /// 
    /// # Arguments
    ///
    /// - `alarm_rate`: The rate at which alarm 2 should trigger. This can be one of several predefined rates in Alarm2Rate.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the alarm rate was successfully set, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
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
    /// 
    /// # Returns
    ///
    /// `Ok(())` if the register was successfully updated, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn update_alarm_1(&mut self) -> Result<(), I2CError> {
        // Update the control/status register. To activate the alarm we want to set bit 0 to 0 
        let last_value = self.read_last_value(CONTROL_STATUS_ADDR)?;
        self.write_clock( last_value & 0xFE , CONTROL_STATUS_ADDR)
    }

    /// Updates the control/status register so the alarm can be activated again. 
    /// 
    /// # Returns
    ///
    /// `Ok(())` if the register was successfully updated, otherwise an `I2CError`.
    /// 
    /// # Errors
    /// 
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn update_alarm_2(&mut self) -> Result<(), I2CError> {
        // Update the control/status register. To activate the alarm we want to set bit 1 to 0 
        let last_value = self.read_last_value(CONTROL_STATUS_ADDR)?;
        self.write_clock( last_value & 0xFD , CONTROL_STATUS_ADDR)
    }

    /// Receives a decimal on two's complement and returns its decimal value.
    ///
    /// # Arguments
    ///
    /// - `value`: The value in two's complement form.
    ///
    /// # Returns
    ///
    /// The decimal equivalent of the two's complement value as f32.
    fn twos_complement_to_decimal(&self, value: u8) -> f32 {
        if value & 0x80 != 0 {
            -((!value + 1) as f32)
        } else {
            value as f32
        }
    }

    /// Returns the temperature on Celsius.
    /// 
    /// # Returns
    ///
    /// The temperature in degrees Celsius as a floating-point value as a f32.
    pub fn get_temperature(&mut self) -> f32 {
        let mut buffer: [u8; 2] = [0; 2];
        self.i2c.write_read(DS3231_ADDR, &[TEMP_ADDR], &mut buffer,BLOCK).unwrap();

        let temp_integer = self.twos_complement_to_decimal(buffer[0]);
        let temp_fractional = self.twos_complement_to_decimal(buffer[1] >> 6) * 0.25;  // We only need the 2 most significant bits of the register

        temp_integer + temp_fractional
    }
}


impl<'a> READER for DS3231<'a> {
    /// Reads the DS3231 registers and parses the data into a 
    /// `HashMap` where each key corresponds to a time component (seconds, minutes, hours, etc.).
    ///
    /// # Returns
    /// 
    /// A `HashMap<String, String>` containing the parsed time components.
    ///
    /// # Example
    /// ```
    /// let date_time_data = self.read_and_parse();
    /// println!("{:?}", date_time_data.get("hrs")); // Prints the current hour as a string.
    /// ```
    fn read_and_parse(& mut self) -> HashMap<String, String> {
        let mut data: [u8; 13] = [0_u8; 13];
        self.i2c.write(DS3231_ADDR, &[0_u8], BLOCK).unwrap();
        self.i2c.read(DS3231_ADDR, &mut data, BLOCK).unwrap();
        self.parse_read_data(data)
    }
}

impl DateTimeComponent {

    /// Gets the register address of the desired DateTime component.
    /// 
    /// # Returns
    /// 
    /// An u8 representing the register address
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
    /// 
    /// # Arguments
    /// 
    /// - `val`: An u8 representing the value to check
    /// 
    /// # Returns
    /// 
    /// A bool. True if val is between boundaries, False if not.
    pub fn is_between_boundaries(&self, val: u8, hour_mode: HourMode) -> bool {
        match self {
            DateTimeComponent::Second => self.check_boundaries(val, MIN_VALUE, MAX_SECS),
            DateTimeComponent::Minute => self.check_boundaries(val, MIN_VALUE, MAX_MINS),
            DateTimeComponent::Hour => {
                match hour_mode {
                    HourMode::TwentyFourHour => self.check_boundaries(val, MIN_VALUE, MAX_24_HOUR_MODE),
                    _ => self.check_boundaries(val, MIN_VALUE, MAX_12_HOUR_MODE),
                }
            },
            DateTimeComponent::WeekDay => self.check_boundaries(val, MIN_WEEK_DAY, MAX_WEEK_DAY),
            DateTimeComponent::Date => self.check_boundaries(val, MIN_VALUE, MAX_DATE),
            DateTimeComponent::Month => self.check_boundaries(val, MIN_MONTH, MAX_MONTH),
            DateTimeComponent::Year => self.check_boundaries(val, MIN_VALUE, MAX_YEAR),
        }
    }

    /// Receives a value and the boundaries. Returstrue if the values is between boundaries, else false.
    /// 
    /// # Arguments
    /// 
    /// - `val`: The value to check.
    /// - `min_boundarie`: The minimum allowable value (inclusive).
    /// - `max_boundarie`: The maximum allowable value (inclusive).
    ///
    /// # Returns
    /// 
    /// True if `val` is between `min_boundarie` and `max_boundarie`, inclusive. False otherwise.
    ///
    /// # Example
    /// ```
    /// let result = self.check_boundaries(5, 1, 10);
    /// assert!(result); // true
    /// ```
    fn check_boundaries(&self, val: u8, min_boundarie: u8, max_boundarie: u8) -> bool {
        val >= min_boundarie && val <= max_boundarie
    }
}

impl HourMode {

    /// Gets the bitmask needed to read hour values according to the hour-mode
    /// 
    /// When reading the hour from the DS3231 the hour-mode changes which bits should be ignored
    /// 
    /// # Returns
    /// 
    /// An u8 that represents the bitmask to be used
    fn get_read_bitmask(&self) -> u8 {
        match self {
            Self::TwentyFourHour => READ_BITMASK_24_HOUR_MODE,
            _  => READ_BITMASK_12_HOUR_MODE
        }
    }

    /// Gets the bitmask needed to write hour values according to the hour-mode
    /// 
    /// When writing the the hour on the DS3231 some bits (apart from the ones holding the hour value)
    /// need to be set according to the hour-mode
    /// 
    /// # Returns
    /// 
    /// An u8 that represents the bitmask to be used
    fn get_write_bitmask(&self) -> u8 {
        match self {
            HourMode::TwentyFourHour => WRITE_BITMASK_24_HOUR_MODE,
            HourMode::TwelveHourAM => WRITE_BITMASK_12_AM_HOUR_MODE,
            HourMode::TwelveHourPM => WRITE_BITMASK_12_PM_HOUR_MODE,
        }
    }
}
