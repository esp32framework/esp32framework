use crate::{
    microcontroller_src::peripherals::{Peripheral, PeripheralError},
    utils::auxiliary::micro_to_ticks,
};
use esp_idf_svc::hal::{
    delay::BLOCK,
    gpio::{Gpio0, Gpio1},
    uart::{config, UartDriver, UART0, UART1},
    units::Hertz,
};

const DEFAULT_BAUDRATE: u32 = 115_200;

/// Error types related to UART operations.
#[derive(Debug)]
pub enum UARTError {
    DriverError,
    InvalidBaudrate,
    InvalidPeripheral(PeripheralError),
    InvalidPin,
    InvalidUartNumber,
    ReadError,
    WriteError,
}

/// Represents the stop bit settings for UART communication.
#[derive(Debug)]
pub enum StopBit {
    One,
    OnePointFive,
    Two,
}

/// Represents the parity settings for UART communication.
#[derive(Debug)]
pub enum Parity {
    Even,
    Odd,
    None,
}

/// A UART (Universal Asynchronous Receiver Transmitter) driver to handle serial communications.
pub struct UART<'a> {
    driver: UartDriver<'a>,
}

impl<'a> UART<'a> {
    /// Creates a new UART driver.
    ///
    /// # Arguments
    ///
    /// - `tx`: The peripheral pin connected to TX.
    /// - `rx`: The peripheral pin connected to RX.
    /// - `uart_peripheral`: The UART peripheral to use.
    /// - `baudrate`: The desired baud rate in bits per second.
    /// - `parity`: The desired parity configuration.
    /// - `stopbit`: The desired stop bit configuration.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `UART` instance, or a `UARTError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::InvalidPin`: If either the TX or RX pins cannot be converted to IO pins.
    /// - `UARTError::InvalidUartNumber`: If an unsupported UART peripheral is selected.
    /// - `UARTError::DriverError`: If there is an error initializing the driver.
    pub(crate) fn new(
        tx: Peripheral,
        rx: Peripheral,
        uart_peripheral: Peripheral,
        baudrate: u32,
        parity: Parity,
        stopbit: StopBit,
    ) -> Result<UART<'a>, UARTError> {
        let rx_peripheral = rx.into_any_io_pin().map_err(UARTError::InvalidPeripheral)?;
        let tx_peripheral = tx.into_any_io_pin().map_err(UARTError::InvalidPeripheral)?;
        let config = set_config(baudrate, parity, stopbit)?;

        let driver = match uart_peripheral {
            Peripheral::Uart(0) => UartDriver::new(
                unsafe { UART0::new() },
                tx_peripheral,
                rx_peripheral,
                Option::<Gpio0>::None,
                Option::<Gpio1>::None,
                &config,
            )
            .map_err(|_| UARTError::DriverError)?,
            Peripheral::Uart(1) => UartDriver::new(
                unsafe { UART1::new() },
                tx_peripheral,
                rx_peripheral,
                Option::<Gpio0>::None,
                Option::<Gpio1>::None,
                &config,
            )
            .map_err(|_| UARTError::DriverError)?,
            _ => return Err(UARTError::InvalidUartNumber),
        };

        Ok(UART { driver })
    }

    /// Creates a UART driver with default baudrate of 115200 Hz, none parity and one bit stop bit.
    ///
    /// # Arguments
    ///
    /// - `tx`: The peripheral pin connected to TX.
    /// - `rx`: The peripheral pin connected to RX.
    /// - `uart_peripheral`: The UART peripheral to use.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `UART` instance, or a `UARTError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::InvalidPin`: If either the TX or RX pins cannot be converted to IO pins.
    /// - `UARTError::InvalidUartNumber`: If an unsupported UART peripheral is selected.
    /// - `UARTError::DriverError`: If there is an error initializing the driver.
    pub fn default(
        tx: Peripheral,
        rx: Peripheral,
        uart_peripheral: Peripheral,
    ) -> Result<UART<'a>, UARTError> {
        UART::new(
            tx,
            rx,
            uart_peripheral,
            DEFAULT_BAUDRATE,
            Parity::None,
            StopBit::One,
        )
    }

    /// Write multiple bytes from a slice. Returns how many bytes were written or an error
    /// if the write operation fails.
    ///
    /// # Arguments
    ///
    /// - `bytes_to_write`: A slice of bytes to write.
    ///
    /// # Returns
    ///
    /// A `Result` with the size of the write data if the operation completed successfully, or
    /// an `UARTError` if it fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::WriteError`: If the write operation failed.
    pub fn write(&mut self, bytes_to_write: &[u8]) -> Result<usize, UARTError> {
        self.driver
            .write(bytes_to_write)
            .map_err(|_| UARTError::WriteError)
    }

    /// Reads from the UART buffer without a timeout. This means that the function will be blocking
    /// until the buffer passed gets full. If the buffer never gets full, the function will never return.
    ///
    /// # Arguments
    ///
    /// - `buffer`: A mutable slice of bytes to store the read data.
    ///
    /// # Returns
    ///
    /// A `Result` with the size of the read data if the operation completed successfully, or
    /// an `UARTError` if it fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::ReadError`: If the read operation failed.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UARTError> {
        self.driver
            .read(buffer, BLOCK)
            .map_err(|_| UARTError::ReadError)
    }

    /// Reads from the UART buffer with a timeout in us (microsec). The function will
    /// return once the timeout is reached or the buffer is full.
    ///
    /// # Arguments
    ///
    /// - `buffer`: A mutable slice of bytes to store the read data.
    ///
    /// # Returns
    ///
    /// A `Result` with the size of the read data if the operation completed successfully, or
    /// an `UARTError` if it fails.
    ///
    /// # Errors
    ///
    /// - `UARTError::ReadError`: If the read operation failed.
    pub fn read_with_timeout(
        &mut self,
        buffer: &mut [u8],
        timeout_us: u32,
    ) -> Result<usize, UARTError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver
            .read(buffer, timeout)
            .map_err(|_| UARTError::ReadError)
    }
}

/// Sets up the UART configuration based on the given parameters.
///
/// # Arguments
///
/// - `baudrate`: The desired baud rate in bits per second.
/// - `parity`: The desired parity configuration.
/// - `stopbit`: The desired stop bit configuration.
///
/// # Returns
///
/// A `Result` containing a config::Config if successful  or an `UARTError` if it fails.
///
/// # Errors
///
/// - `UARTError::BaudrateConversionError`: If there's an issue converting the baudrate to Hz.
fn set_config(
    baudrate: u32,
    parity: Parity,
    stopbit: StopBit,
) -> Result<config::Config, UARTError> {
    let bd = baudrate_to_hertz(baudrate)?;
    let mut config = config::Config::new().baudrate(bd);
    config = match parity {
        Parity::Even => config.parity_even(),
        Parity::Odd => config.parity_odd(),
        Parity::None => config.parity_none(),
    };
    config = match stopbit {
        StopBit::One => config.stop_bits(config::StopBits::STOP1),
        StopBit::OnePointFive => config.stop_bits(config::StopBits::STOP1P5),
        StopBit::Two => config.stop_bits(config::StopBits::STOP2),
    };
    Ok(config)
}

/// Converts a baudrate value to its corresponding Hertz frequency.
///
/// # Parameters
///
/// * `baudrate`: The input baudrate value.
///
/// # Returns
///
/// A `Result` containing a Hertz(freq) if successful  or an `UARTError` if it fails.
///
/// # Errors
///
/// - `UARTError::InvalidBaudrate`: If there's an issue converting the baudrate to Hz.
fn baudrate_to_hertz(baudrate: u32) -> Result<Hertz, UARTError> {
    match baudrate {
        110 => Ok(Hertz(110)),
        300 => Ok(Hertz(300)),
        600 => Ok(Hertz(600)),
        1200 => Ok(Hertz(1200)),
        2400 => Ok(Hertz(2400)),
        4800 => Ok(Hertz(4800)),
        9600 => Ok(Hertz(9600)),
        14400 => Ok(Hertz(14400)),
        19200 => Ok(Hertz(19200)),
        38400 => Ok(Hertz(38400)),
        57600 => Ok(Hertz(57600)),
        115200 => Ok(Hertz(115200)),
        128000 => Ok(Hertz(128000)),
        256000 => Ok(Hertz(256000)),
        460800 => Ok(Hertz(460800)),
        921600 => Ok(Hertz(921600)),
        1843200 => Ok(Hertz(1843200)),
        3686400 => Ok(Hertz(3686400)),
        _ => Err(UARTError::InvalidBaudrate),
    }
}
