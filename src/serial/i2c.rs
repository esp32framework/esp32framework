use std::collections::HashMap;
use esp_idf_svc::{hal::{i2c::{I2cConfig, I2cDriver, I2cSlaveConfig, I2cSlaveDriver, I2C0}, units::FromValueType}, sys::{ESP_ERR_INVALID_ARG, ESP_ERR_NO_MEM, ESP_ERR_TIMEOUT}};
use crate::microcontroller::peripherals::Peripheral;
use super::micro_to_ticks;

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

    pub fn read(&mut self, addr: u8, buffer: &mut [u8], timeout_us: u32) -> Result<(), I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.read(addr, buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

    pub fn write(&mut self, addr: u8, bytes_to_write: &[u8], timeout_us: u32) -> Result<(), I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.write(addr, bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

    pub fn write_read(&mut self, addr: u8, bytes_to_write: &[u8], buffer: &mut [u8], timeout_us: u32) -> Result<(), I2CError>{
        let timeout: u32 = micro_to_ticks(timeout_us);
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

    pub fn read(&mut self, buffer: &mut [u8], timeout_us: u32) -> Result<usize, I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.read(buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_TIMEOUT => I2CError::TimeoutError,
            _ => I2CError::InvalidArg,
        })
    }

    pub fn write(&mut self, bytes_to_write: &[u8], timeout_us: u32) -> Result<usize, I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.write(bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_TIMEOUT => I2CError::TimeoutError,
            _ => I2CError::InvalidArg,
        })
    }

}