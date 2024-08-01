use std::collections::HashMap;
use esp_idf_svc::{hal::{delay::{Delay, FreeRtos}, i2c::{I2cConfig, I2cDriver, I2cSlaveConfig, I2cSlaveDriver, I2C0}, modem::WifiModemPeripheral, units::FromValueType}, sys::{esp_sleep_mode_t_ESP_SLEEP_MODE_LIGHT_SLEEP, ESP_ERR_INVALID_ARG, ESP_ERR_NO_MEM, ESP_ERR_TIMEOUT}};
use crate::microcontroller::peripherals::Peripheral;


const DEFAULT_BAUDRATE: u32 = 100;

#[derive(Debug)]
pub enum I2CError {
    Temp,
    InvalidPin,
    InvalidPeripheral,
    BufferTooSmall,
    InvalidArg,
    DriverError,
    NoMoreHeapMemory,
    TimeoutError,
    ErrorInReadValue
}

pub struct I2CMaster<'a> {
    driver: I2cDriver<'a>,
}

impl <'a>I2CMaster<'a> {
    pub fn new(sda_per: Peripheral, scl_per: Peripheral, i2c: I2C0) -> Result<I2CMaster<'a>, I2CError> { // TODO: What can we do with i2c_per
        let sda = sda_per.into_any_io_pin().map_err(|_| I2CError::InvalidPin)?;
        let scl = scl_per.into_any_io_pin().map_err(|_| I2CError::InvalidPin)?;

        let config = I2cConfig::new().baudrate(DEFAULT_BAUDRATE.kHz().into());
        let driver = I2cDriver::new(i2c, sda, scl, &config).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            _ => I2CError::DriverError, 
        })?;

        Ok(
            I2CMaster { driver }
        )
    }

    pub fn read(&mut self, addr: u8, buffer: &mut [u8], timeout: u32) -> Result<(), I2CError> {
        self.driver.read(addr, buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

    pub fn write(&mut self, addr: u8, bytes_to_write: &[u8], timeout: u32) -> Result<(), I2CError> {
        self.driver.write(addr, bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

    pub fn write_read(&mut self, addr: u8, bytes_to_write: &[u8], buffer: &mut [u8], timeout: u32) -> Result<(), I2CError>{
        self.driver.write_read(addr, bytes_to_write, buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

}

pub struct I2CSlave<'a> {
    driver: I2cSlaveDriver<'a>
}

impl <'a>I2CSlave<'a> {

    pub fn new(sda_per: Peripheral, scl_per: Peripheral, i2c: I2C0, addr: u8) -> Result<I2CSlave<'a>, I2CError> {
        let sda = sda_per.into_any_io_pin().map_err(|_| I2CError::InvalidPin)?;
        let scl = scl_per.into_any_io_pin().map_err(|_| I2CError::InvalidPin)?;

        let config = I2cSlaveConfig::new(); // TODO: Check if the default values work. It has the buffers on 0. Maybe this should be choosen by the user
        let driver = I2cSlaveDriver::new(i2c, sda, scl, addr, &config).unwrap();

        Ok(
            I2CSlave { driver }
        )
    }

    pub fn read(&mut self, buffer: &mut [u8], timeout: u32) -> Result<usize, I2CError> {
        self.driver.read(buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_TIMEOUT => I2CError::TimeoutError,
            _ => I2CError::InvalidArg,
        })
    }

    pub fn write(&mut self, bytes_to_write: &[u8], timeout: u32) -> Result<usize, I2CError> {
        self.driver.write(bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_TIMEOUT => I2CError::TimeoutError,
            _ => I2CError::InvalidArg,
        })
    }

}

pub trait READER {

    fn read_and_parse<'b>(&'b mut self) -> HashMap<String,String>;

    fn show_data(&mut self, operation_key: String) -> Result<(), I2CError> {
        let parsed_data: HashMap<String, String> = self.read_and_parse();
        match parsed_data.get(&operation_key) {
            Some(data) => println!("The content is: {:?}", data),
            None => {return Err(I2CError::ErrorInReadValue);}
        }
        Ok(())
    }

    // TODO: Document that we work with floats (values with "," will explode)
    fn read_n_times_and_sum(&mut self, operation_key: String, times: usize, ms_between_reads: u32) -> Result<f32, I2CError> {
        let mut total = 0.0;
        for _ in 0..times {
            let parsed_data: HashMap<String, String> = self.read_and_parse();
            match parsed_data.get(&operation_key) {
                Some(data) =>  total += data.parse::<f32>().map_err(|_| I2CError::ErrorInReadValue)?,
                None => {return Err(I2CError::ErrorInReadValue)}
            }
            FreeRtos::delay_ms(ms_between_reads);
        }
        Ok(total)
    }

    // TODO: Document that we work with floats (values with "," will explode)
    fn read_n_times_and_avg(&mut self, operation_key: String, times: usize, ms_between_reads: u32) -> Result<f32, I2CError> {
        let mut total = 0.0;
        for _ in 0..times {
            let parsed_data: HashMap<String, String> = self.read_and_parse();
            match parsed_data.get(&operation_key) {
                Some(data) =>  total += data.parse::<f32>().map_err(|_| I2CError::ErrorInReadValue)?,
                None => {return Err(I2CError::ErrorInReadValue)}
            }
            FreeRtos::delay_ms(ms_between_reads);
        }
        Ok((total as f32) / (times as f32))
    }

    fn read_n_times_and_aggregate<C, T>(&mut self, operation_key: String, times: usize, ms_between_reads: u32, execute_closure: C) -> Result<T, I2CError>
    where
    C: Fn(Vec<String>) -> T
    {
        let mut read_values: Vec<String> = vec![];
        for _ in 0..times {
            let parsed_data: HashMap<String, String> = self.read_and_parse();
            match parsed_data.get(&operation_key) {
                Some(data) =>  {
                    println!("{:?}", data);
                    read_values.push(data.clone());
                },
                None => {return Err(I2CError::ErrorInReadValue)}
            }
            FreeRtos::delay_ms(ms_between_reads);
        }
        Ok(execute_closure(read_values))
    }

    fn execute_when_true<C1, C2>(&mut self, operation_key: String, ms_between_reads: u32, condition_closure: C1, execute_closure: C2) -> Result<(), I2CError> 
    where
    C1: Fn(String) -> bool,
    C2: Fn(HashMap<String, String>) -> (),
    {
        loop {
            let parsed_data: HashMap<String, String> = self.read_and_parse();
            match parsed_data.get(&operation_key) {
                Some(data) =>  {
                    if condition_closure(data.clone()) {
                        execute_closure(parsed_data);
                    }
                },
                None => {return Err(I2CError::ErrorInReadValue)}
            }
            FreeRtos::delay_ms(ms_between_reads);
        }
    }
}

pub trait WRITER { 
    fn parse_and_write(&mut self, addr: u8, bytes_to_write: &[u8]) -> Result<(), I2CError>;
    
    fn write_with_frecuency(&mut self, ms_between_writes: u32, addr: u8, bytes_to_write: &[u8]) -> Result<(), I2CError> {
        loop{
            if let Err(e) = self.parse_and_write(addr, bytes_to_write) {
                return Err(e);
            }
            FreeRtos::delay_ms(ms_between_writes);
        }
    }
}

// TODO: Check this name
pub trait ReadWriter: WRITER + READER {

    fn write_when_true(&mut self, operation_key: String, ms_between_reads: u32, addr: u8, bytes_to_write: &[u8]) -> Result<(), I2CError> { 
        loop {
            let parsed_data: HashMap<String, String> = self.read_and_parse();
            match parsed_data.get(&operation_key) {
                Some(_) => {
                    if let Err(e) = self.parse_and_write(addr, bytes_to_write) {
                        return Err(e);
                    }
                },
                None => {}
            }
            FreeRtos::delay_ms(ms_between_reads);
        }
    }
}


