use esp_idf_svc::hal::{delay::BLOCK, uart::{config, UartDriver, UART0, UART1}, units::{FromValueType, Hertz}};
use crate::microcontroller::peripherals::Peripheral;
use esp_idf_svc::hal::gpio::{Gpio0, Gpio1};

use super::micro_to_ticks;


const DEFAULT_BAUDRATE: u32 = 115;

#[derive(Debug)]
pub enum UARTError{
    InvalidPin,
    InvalidUartNumber,
    WriteError,
    ReadError,
}


pub struct UART<'a> {
    driver: UartDriver<'a>,
}

impl <'a>UART<'a> {
    pub fn new(tx: Peripheral, rx: Peripheral, uart_peripheral: Peripheral) -> Result<UART<'a>, UARTError > {
        let rx_peripheral = rx.into_any_io_pin().map_err(|_| UARTError::InvalidPin)?;
        let tx_peripheral = tx.into_any_io_pin().map_err(|_| UARTError::InvalidPin)?;
        let config = config::Config::new().baudrate(Hertz(115_200));
        
        let driver = match uart_peripheral {
            Peripheral::UART(0) => {UartDriver::new(
                unsafe{ UART0::new()},
                tx_peripheral,
                rx_peripheral,
                Option::<Gpio0>::None,
                Option::<Gpio1>::None,
                &config,
            ).unwrap()},
            Peripheral:: UART(1) => {
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