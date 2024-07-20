use esp_idf_svc::{hal::{gpio::{InputPin, OutputPin}, i2c::{self, I2cConfig, I2cDriver}, units::FromValueType}, sys::{ESP_ERR_INVALID_ARG, ESP_ERR_NO_MEM}};
use crate::microcontroller::peripherals::Peripheral;



const DEFAULT_BAUDRATE: u32 = 100;

#[derive(Debug)]
pub enum I2CMasterError {
    Temp,
    InvalidPin,
    InvalidPeripheral,
    BufferTooSmall,
    InvalidArg,
    DriverError,
    NoMoreHeapMemory
}

pub struct I2CMaster<'a> {
    driver: I2cDriver<'a>,
}

impl <'a>I2CMaster<'a> {
    pub fn new(sda_per: Peripheral, scl_per: Peripheral, i2c_per: Peripheral) -> Result<I2CMaster<'a>, I2CMasterError> { // TODO: What can we do with i2c_per
        let sda = sda_per.into_any_io_pin().map_err(|_| I2CMasterError::InvalidPin)?;
        let scl = scl_per.into_any_io_pin().map_err(|_| I2CMasterError::InvalidPin)?;

        let config = I2cConfig::new().baudrate(DEFAULT_BAUDRATE.kHz().into());

        let i2c = unsafe{i2c::I2C0::new()};
        let driver = I2cDriver::new(i2c, sda, scl, &config).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CMasterError::InvalidArg,
            ESP_FAIL => I2CMasterError::DriverError, 
        })?;

        Ok(
            I2CMaster { driver }
        )
    }

    pub fn read(&mut self, addr: u8, buffer: &mut [u8], timeout: u32) -> Result<(), I2CMasterError> {
        self.driver.read(addr, buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CMasterError::InvalidArg,
            ESP_ERR_NO_MEM => I2CMasterError::BufferTooSmall,
            ESP_FAIL => I2CMasterError::NoMoreHeapMemory,
        })
    }

    pub fn write(&mut self, addr: u8, bytes_to_write: &[u8], timeout: u32) -> Result<(), I2CMasterError> {
        self.driver.write(addr, bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CMasterError::InvalidArg,
            ESP_ERR_NO_MEM => I2CMasterError::BufferTooSmall,
            ESP_FAIL => I2CMasterError::NoMoreHeapMemory,
        })
    }

    // pub fn default(baudrate: u32){
    //     let config = I2cConfig::new().baudrate(DEFAULT_BAUDRATE.kHz().into());
        
    //     Self::new() 
    // }
}

