//! Example using pin GPIO5 as analog in to read the intensity of a signal.
//! This value is obtained by reading 'SAMPLING_QUANTITY' times and then 
//! printing its average.

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::adc::{oneshot::{AdcChannelDriver, AdcDriver,config::AdcChannelConfig}, attenuation::DB_11};

const SAMPLING_QUANTITY: u16 = 10;

fn main() {
    let peripherals = Peripherals::take().unwrap();
    let adc_driver = AdcDriver::new(peripherals.adc1).unwrap();

    let config = AdcChannelConfig {
        attenuation: DB_11,
        calibration: true,
        ..Default::default()
    };

    let mut adc_pin = AdcChannelDriver::new(&adc_driver, peripherals.pins.gpio5, &config).unwrap();

    loop {
        let mut smooth_read = 0;
        for _ in 0..SAMPLING_QUANTITY {
            smooth_read += adc_pin.read().unwrap();
        }
        smooth_read /= SAMPLING_QUANTITY;

        println!("ADC value with {:?} amount of reads: {:?}",SAMPLING_QUANTITY, smooth_read);

        FreeRtos::delay_ms(1000);
    }
}