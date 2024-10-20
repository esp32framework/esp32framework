use esp_idf_svc::hal::delay::BLOCK;
use crate::serial::i2c::I2CMaster;

/*
unsupported registers:
- config
*/

const RESET_ADDR: u8 = 0xE0;
const CTRL_HUM_ADDR: u8 = 0xF2;
const CTRL_MEAS_ADDR: u8 = 0xF4;
const STATUS_ADDR: u8 = 0xF3;

const CRTL_MEAS_MODE_MASK: u8 = 0b11111100;
const CRTL_MEAS_PRESS_OVS_MASK: u8 = 0b11100011;
const CRTL_MEAS_TEMP_OVS_MASK: u8 = 0b00011111;
const STAUTS_MEASURING_MASK: u8 = 0b00001000;
const STAUTS_UPDATING_MASK: u8 = 0b00000001;

const RESET_WORD: u8 = 0xB6;

/// Enums the possible oversampling options for measurments
pub enum Oversampling {
    X1 = 0x01,  
    X2 = 0x02,  
    X4 = 0x03,  
    X8 = 0x04,  
    X16 = 0x05,
}

/// Enums both of the possible connections of the SDO pin.
/// This is important because it determines the sensors address.
#[derive(Clone, Copy)]
pub enum SdoConnection {
    GND = 0x76,
    VDDIO = 0x77,
}

/// Enums the possible sensor modes:
/// - `Sleep`: No operation, all registers accessible, lowest power, selected after startup.
/// - `Forced`: Perform one measurement, store results and return to sleep mode.
/// - `Normal`: Perpetual cycling of measurements and inactive periods.
pub enum BMEMode {
    Sleep = 0x00,
    Forced = 0x01, // 0x02 could also be used
    Normal = 0x03,
}


pub struct BME280<'a> {
    i2c: I2CMaster<'a>,
    sdo_connection: SdoConnection,
}

impl <'a>BME280<'a> {

    /// Creates a new BME280 instance.
    /// 
    /// # Arguments
    /// 
    /// - `i2c`: The I2CMaster interface to communicate with the BME280.
    /// - `sdo_connection`: The type of connection on the SDO pin.
    /// 
    /// # Returns
    /// 
    /// The new `BME280` instance
    pub fn new(i2c: I2CMaster<'a>, sdo_connection: SdoConnection) -> BME280<'a> {
        BME280 { i2c, sdo_connection}
    }

    fn addr(&self) -> u8 {
        self.sdo_connection as u8
    } 

    pub fn reset(&mut self) {
        self.i2c.write(self.addr(), &[RESET_ADDR, RESET_WORD], BLOCK).unwrap();
    }

    pub fn is_measuring(&mut self) -> bool {
        self.check_status(STAUTS_MEASURING_MASK)
    }

    pub fn is_updating(&mut self) -> bool {
        self.check_status(STAUTS_UPDATING_MASK)
    }

    fn check_status(&mut self, bitmask: u8) -> bool {
        let mut buf: [u8; 1] = [0];
        self.i2c.write_read(self.addr(), &[STATUS_ADDR], &mut buf, BLOCK).unwrap();
        let mut register_val = buf[0];
        register_val &= bitmask;
        
        register_val == bitmask 
    }

    fn read_last_value(&mut self, register_addr: u8) -> u8 {
        
        let mut buf: [u8; 1] = [0];
        self.i2c.write_read(self.addr(), &[register_addr], &mut buf, BLOCK).unwrap();
        buf[0]
    }
    
    pub fn set_hum_oversampling(&mut self, oversampling: Oversampling) {
        // CRTL_HUM and CRTL_MEAS work together. For CRTL_HUM to be effective, CRTL_MEAS needs to be written too,
        // and vice versa. This is explained on BMP280 datasheet section 5.4.3 and 5.4.5.

        // CRTL_HUM
        self.i2c.write(self.addr(), &[CTRL_HUM_ADDR, oversampling as u8], BLOCK).unwrap();
        // CRTL_MEAS
        let value = self.read_last_value(CTRL_MEAS_ADDR);
        self.i2c.write(self.addr(), &[CTRL_MEAS_ADDR, value], BLOCK).unwrap();
    }

    pub fn set_temp_oversampling(&mut self, oversampling: Oversampling) {
        self.set_oversampling(CRTL_MEAS_TEMP_OVS_MASK, oversampling);
    }

    pub fn set_press_oversampling(&mut self, oversampling: Oversampling) {
        self.set_oversampling(CRTL_MEAS_PRESS_OVS_MASK, oversampling);
    }

    pub fn set_mode(&mut self, mode: BMEMode) {
        // CRTL_HUM
        self.rewrite_crtl_hum();
        
        // CRTL_MEAS
        let mut value = self.read_last_value(CTRL_MEAS_ADDR);
        value &= CRTL_MEAS_MODE_MASK;
        value |= mode as u8;
        self.i2c.write(self.addr(), &[CTRL_MEAS_ADDR, value], BLOCK).unwrap();
    }

    fn set_oversampling(&mut self, bitmask: u8, oversampling: Oversampling) {
        // CRTL_HUM
        self.rewrite_crtl_hum();

        // CRTL_MEAS
        let mut value = self.read_last_value(CTRL_MEAS_ADDR);
        self.mask_and_shift(bitmask, &mut value, oversampling);
        self.i2c.write(self.addr(), &[CTRL_MEAS_ADDR, value], BLOCK).unwrap();
    }
    
    /// Gets the value on the CRTL_HUM register and writes it again.
    /// 
    /// This is used on CRTL_MEAS operations. The BME280 datasheet explains that for the changes on
    /// register to be effective, the CRTL_HUM needs to be written before. 
    fn rewrite_crtl_hum(&mut self) {
        let value = self.read_last_value(CTRL_HUM_ADDR);
        self.i2c.write(self.addr(), &[CTRL_HUM_ADDR, value], BLOCK).unwrap();
    }

    fn mask_and_shift(&mut self, mask: u8, value: &mut u8, oversampling: Oversampling) {
        *value &= mask;
        let shifted_oversampling = (oversampling as u8) << 5;
        *value |= shifted_oversampling;
    }

    

}
