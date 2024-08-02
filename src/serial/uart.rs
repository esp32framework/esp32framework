use esp_idf_svc::hal::{uart::{config, UartDriver, UART0, UART1}, units::{FromValueType, Hertz}};
use crate::microcontroller::peripherals::Peripheral;
use esp_idf_svc::hal::gpio::{Gpio0, Gpio1};


const DEFAULT_BAUDRATE: u32 = 115;

#[derive(Debug)]
pub enum UARTError{
    InvalidPin,
    InvalidUartNumber,
    WriteError,
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
}