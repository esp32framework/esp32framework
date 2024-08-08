use esp_idf_svc::hal::{delay::BLOCK, uart::{config, UartDriver, UART0, UART1}, units::Hertz};
use crate::microcontroller_src::peripherals::Peripheral;
use esp_idf_svc::hal::gpio::{Gpio0, Gpio1};

use super::micro_to_ticks;


const DEFAULT_BAUDRATE: u32 = 115_200;

#[derive(Debug)]
pub enum UARTError{
    InvalidPin,
    InvalidUartNumber,
    WriteError,
    ReadError,
    InvalidBaudrate
}

pub enum StopBit {
    One,
    OnePointFive,
    Two,
}

pub enum Parity{
    Even,
    Odd,
    None,
}

pub struct UART<'a> {
    driver: UartDriver<'a>,
}

impl <'a>UART<'a> {
    pub fn new(tx: Peripheral, rx: Peripheral, uart_peripheral: Peripheral, baudrate: u32, parity: Parity, stopbit: StopBit) -> Result<UART<'a>, UARTError > {
        let rx_peripheral = rx.into_any_io_pin().map_err(|_| UARTError::InvalidPin)?;
        let tx_peripheral = tx.into_any_io_pin().map_err(|_| UARTError::InvalidPin)?;
        let config = set_config(baudrate, parity, stopbit)?;

        let driver = match uart_peripheral {
            Peripheral::Uart(0) => {UartDriver::new(
                unsafe{ UART0::new()},
                tx_peripheral,
                rx_peripheral,
                Option::<Gpio0>::None,
                Option::<Gpio1>::None,
                &config,
            ).unwrap()},
            Peripheral::UART(1) => {
                UartDriver::new(
                    unsafe{ UART1::new()},
                    tx_peripheral,
                    rx_peripheral,
                    Option::<Gpio0>::None,
                    Option::<Gpio1>::None,
                    &config,
                ).unwrap()
            },
            _ => return Err(UARTError::InvalidUartNumber),
        };
        
        Ok(UART{driver})
    }
    
    /// Returns a UART with default baud rate of 115200 Hz, none parity and one bit stop bit.
    pub fn default(tx: Peripheral, rx: Peripheral, uart_peripheral: Peripheral) -> Result<UART<'a>, UARTError > {
        UART::new(tx,rx,uart_peripheral, DEFAULT_BAUDRATE, Parity::None, StopBit::One)
    }
    
    /// Write multiple bytes from a slice. Returns how many bytes were written or an error 
    /// if the write operation fails.
    pub fn write(&mut self, bytes_to_write: &[u8]) -> Result<usize, UARTError> {
        self.driver.write(bytes_to_write).map_err(|_| UARTError::WriteError)
    }

    /// Reads from the UART buffer without a timeout. This means that the function will be blocking 
    /// until the buffer passed gets full. If the buffer never gets full, the function will never return.
    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, UARTError> {
        self.driver.read(buffer, BLOCK).map_err(|_| UARTError::ReadError)
    }

    /// Reads from the UART buffer with a timeout in us (microsec). The function will return once the timeout is reached or the buffer is full.
    pub fn read_with_timeout(&mut self, buffer: &mut [u8], timeout_us: u32) -> Result<usize, UARTError> {
        let timeout: u32 = micro_to_ticks(timeout_us);
        self.driver.read(buffer, timeout).map_err(|_| UARTError::ReadError)
    }
}

fn set_config(baudrate: u32, parity: Parity, stopbit: StopBit) -> Result<config::Config, UARTError >{
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

fn baudrate_to_hertz(baudrate: u32) -> Result<Hertz, UARTError> {
    match baudrate{
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