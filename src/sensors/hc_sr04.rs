use std::sync::{atomic::AtomicU32, Arc};
use esp_idf_svc::{hal::delay::Delay, sys::esp_timer_get_time};
use crate::gpio::{DigitalIn, DigitalOut, DigitalOutError};


const SOUND_SPEED_M_S: f64 = 340.0;
const SOUND_SPEED_CM_US: f64 = SOUND_SPEED_M_S * 100.0 / 1_000_000.0;


/// Simple abstraction of the HCSR04 that facilitates its handling
pub struct HCSR04<'a> {
    trig: DigitalOut<'a>,
    echo: DigitalIn<'a>,
}

impl <'a>HCSR04<'a> {

    /// Creates a new I2C master driver.
    /// 
    /// # Arguments
    /// 
    /// - `trig`: A DigitalOut pin that is used to trigger the ultrasound
    /// - `echo`: A DigitalIn pin that is used to receive the echo of the ultrasound
    /// 
    /// # Returns
    /// 
    /// The new HCSR04 instance
    pub fn new(trig: DigitalOut<'a>, echo: DigitalIn<'a>) -> HCSR04<'a> {
        let _echo_ans_time = Arc::new(AtomicU32::new(0));
        HCSR04 { trig, echo}
    }

    /// Returns the distance of the object in front of the sensor in centimeters
    /// 
    /// This function is blocking, since it has to wait for the echo of the ultrasound to get back.
    /// 
    /// # Returns
    /// 
    /// A `Result` containing the an f64 representing the distance in centimeters, or a `DigitalOutError` if the
    /// reading fails.
    /// 
    /// # Errors
    /// 
    /// - `DigitalOutError::InvalidPin`: If the trigger pin level cannot be set.
    pub fn get_distance(&mut self) -> Result<f64, DigitalOutError> {
        let delay = Delay::new_default();
        
        // First set the trigger to Low for a few micro-seconds to get a clean signal
        // Then set the trigger pin high for 10 micro-seconds to send the sonic burst
        self.trig.set_low()?;
        delay.delay_us(4);
        self.trig.set_high()?;
        delay.delay_us(10);
        self.trig.set_low()?;
        
        while self.echo.is_low() {
            // Nothing
        }
        let send_echo_time = unsafe { esp_timer_get_time() };

        while self.echo.is_high() {
            // Nothing
        }
        let rec_echo_time = unsafe { esp_timer_get_time() };
        
        let travel_time = rec_echo_time - send_echo_time;
        let cm: f64 = SOUND_SPEED_CM_US * travel_time as f64;
        Ok(cm / 2.0) // We divide by 2 because if not we get the distance of the roundtrip
    }

}
