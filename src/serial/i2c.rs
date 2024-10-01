use esp_idf_svc::{
    hal::{
        i2c::{I2cConfig, I2cDriver, I2cSlaveConfig, I2cSlaveDriver}, 
        units::FromValueType
    }, 
    sys::{ESP_ERR_INVALID_ARG, ESP_ERR_NO_MEM, ESP_ERR_TIMEOUT}
};
use crate::{
    microcontroller_src::peripherals::{Peripheral, PeripheralError}, 
    utils::auxiliary::micro_to_ticks
};


const DEFAULT_BAUDRATE: u32 = 100;

/// Error types related to I2C operations.
#[derive(Debug)]
pub enum I2CError {
    BufferTooSmall,
    DriverError,
    ErrorInReadValue,
    InvalidArg,
    InvalidPeripheral,
    InvalidPin,
    NoMoreHeapMemory,
    PeripheralError(PeripheralError),
    Temp,
    TimeoutError,
}


/// An I2C master driver of an I2C communication.
pub struct I2CMaster<'a> {
    driver: I2cDriver<'a>,
}

impl <'a>I2CMaster<'a> {
    /// Creates a new I2C master driver.
    ///
    /// # Arguments
    ///
    /// - `sda_per`: The peripheral pin connected to SDA.
    /// - `scl_per`: The peripheral pin connected to SCL.
    /// - `i2c`: The I2C bus to use. ESP32 C6 only has I2C0 bus.
    ///
    /// # Returns
    /// 
    /// A `Result` containing the new `I2CMaster` instance, or an `I2CError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidPin`: If either the SDA or SCL pins cannot be converted to IO pins.
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::DriverError`: If there is an error initializing the driver.
    pub fn new(sda_per: Peripheral, scl_per: Peripheral, i2c_per: Peripheral) -> Result<I2CMaster<'a>, I2CError> {
        let sda = sda_per.into_any_io_pin().map_err(I2CError::PeripheralError)?;
        let scl = scl_per.into_any_io_pin().map_err(I2CError::PeripheralError)?;
        let i2c = i2c_per.into_i2c0().map_err(I2CError::PeripheralError)?;

        let config = I2cConfig::new().baudrate(DEFAULT_BAUDRATE.kHz().into());
        let driver = I2cDriver::new(i2c, sda, scl, &config).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            _ => I2CError::DriverError, 
        })?;

        Ok(
            I2CMaster { driver }
        )
    }

    /// Reads data from the specified address into the provided buffer with a timeout in us (microsec). The function 
    /// will return once the timeout is reached or the buffer is full.
    ///
    /// # Arguments
    ///
    /// - `addr`: The 7-bit address of the I2C slave device.
    /// - `buffer`: A mutable slice of bytes to store the read data.
    /// - `timeout_us`: The maximum duration in microseconds to wait for the operation to complete.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the read operation completed successfully, or an `I2CError` if it fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn read(&mut self, addr: u8, buffer: &mut [u8], timeout_us: u32) -> Result<(), I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.read(addr, buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

    /// Write multiple bytes from a slice to the specified address with a timeout in us (microsec).
    ///
    /// # Arguments
    ///
    /// - `addr`: The 7-bit address of the I2C slave device.
    /// - `bytes_to_write`: A slice of bytes to write.
    /// - `timeout_us`: The maximum duration in microseconds to wait for the operation to complete.
    ///
    /// # Returns
    /// 
    /// A `Result` with Ok if the operation completed successfully, or an `I2CError` if it fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn write(&mut self, addr: u8, bytes_to_write: &[u8], timeout_us: u32) -> Result<(), I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.write(addr, bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

    /// Writes multiple bytes from a slice to the specified address and then reads the answer and stores it into the 
    /// provided buffer.
    ///
    /// # Arguments
    ///
    /// - `addr`: The 7-bit address of the I2C slave device.
    /// - `bytes_to_write`: A slice of bytes to write.
    /// - `timeout_us`: The maximum duration in microseconds to wait for the operation to complete.
    ///
    /// # Returns
    /// 
    /// A `Result` with Ok if the operation completed successfully, or an `I2CError` if it fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::BufferTooSmall`: If the buffer is too small.
    /// - `I2CError::NoMoreHeapMemory`: If there isn't enough heap memory to perform the operation.
    pub fn write_read(&mut self, addr: u8, bytes_to_write: &[u8], buffer: &mut [u8], timeout_us: u32) -> Result<(), I2CError>{
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.write_read(addr, bytes_to_write, buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_INVALID_ARG => I2CError::InvalidArg,
            ESP_ERR_NO_MEM => I2CError::BufferTooSmall,
            _ => I2CError::NoMoreHeapMemory,
        })
    }

}

/// An I2C slave driver that responds to I2C master devices.
pub struct I2CSlave<'a> {
    driver: I2cSlaveDriver<'a>
}

impl <'a>I2CSlave<'a> {
    /// Creates a new I2C slace driver.
    ///
    /// # Arguments
    ///
    /// - `sda_per`: The peripheral pin connected to SDA.
    /// - `scl_per`: The peripheral pin connected to SCL.
    /// - `i2c`: The I2C bus to use. ESP32 C6 only has I2C0 bus.
    /// - `addr`: The 7-bit address of the I2C slave device.
    ///
    /// # Returns
    /// 
    /// A `Result` containing the new `I2CSlave` instance, or an `I2CError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidPin`: If either the SDA or SCL pins cannot be converted to IO pins.
    pub fn new(sda_per: Peripheral, scl_per: Peripheral, i2c_per: Peripheral, addr: u8) -> Result<I2CSlave<'a>, I2CError> {
        let sda = sda_per.into_any_io_pin().map_err(I2CError::PeripheralError)?;
        let scl = scl_per.into_any_io_pin().map_err(I2CError::PeripheralError)?;
        let i2c = i2c_per.into_i2c0().map_err(I2CError::PeripheralError)?;
        
        let config = I2cSlaveConfig::new(); // TODO: Check if the default values work. It has the buffers on 0. Maybe this should be choosen by the user
        let driver = I2cSlaveDriver::new(i2c, sda, scl, addr, &config).unwrap();

        Ok(
            I2CSlave { driver }
        )
    }

    /// Reads data from the specified address into the provided buffer with a timeout in us (microsec). The function 
    /// will return once the timeout is reached or the buffer is full.
    ///
    /// # Arguments
    ///
    /// - `buffer`: A mutable slice of bytes to store the read data.
    /// - `timeout_us`: The maximum duration in microseconds to wait for the operation to complete.
    ///
    /// # Returns
    /// 
    /// A `Result` with the size of the read data if the operation completed successfully, or 
    /// an `I2CError` if it fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::TimeoutError`: If the operation exceeded the specified timeout.
    pub fn read(&mut self, buffer: &mut [u8], timeout_us: u32) -> Result<usize, I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.read(buffer, timeout).map_err(|error| match error.code() {
            ESP_ERR_TIMEOUT => I2CError::TimeoutError,
            _ => I2CError::InvalidArg,
        })
    }
    
    /// Write multiple bytes from a slice with a timeout in us (microsec).
    ///
    /// # Arguments
    ///
    /// - `addr`: The 7-bit address of the I2C slave device.
    /// - `bytes_to_write`: A slice of bytes to write.
    /// - `timeout_us`: The maximum duration in microseconds to wait for the operation to complete.
    ///
    /// # Returns
    /// 
    /// A `Result` with Ok and how many bytes were written if the operation completed successfully, 
    /// or an `I2CError` if it fails.
    ///
    /// # Errors
    ///
    /// - `I2CError::InvalidArg`: If an invalid argument is passed.
    /// - `I2CError::TimeoutError`: If the operation exceeded the specified timeout.
    pub fn write(&mut self, bytes_to_write: &[u8], timeout_us: u32) -> Result<usize, I2CError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.write(bytes_to_write, timeout).map_err(|error| match error.code() {
            ESP_ERR_TIMEOUT => I2CError::TimeoutError,
            _ => I2CError::InvalidArg,
        })
    }

}
