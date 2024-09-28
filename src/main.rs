
use std::time::{Duration, Instant};

use esp_idf_svc::{hal::{adc::{attenuation, oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver}, Resolution, ADC1}, delay::FreeRtos, gpio::Gpio5, prelude::Peripherals}, sys::EspError};

fn main() {

    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    
    let adc_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let mut config = AdcChannelConfig::new();
    config.attenuation = attenuation::NONE;
    config.resolution = Resolution::Resolution12Bit;
    config.calibration = true;
    
    let pin = peripherals.pins.gpio5;
    let mut sound_in = AdcChannelDriver::new(adc_driver, pin, &config).unwrap();

    FreeRtos::delay_ms(2000);
    let first_second = smooth_read_during(&mut sound_in, 1000).unwrap() as f32;
    println!("Value of first second #{first_second}");
    FreeRtos::delay_ms(2000);
    
    for _ in 0..10{
        let sound = smooth_read_during(&mut sound_in, 1000).unwrap() as f32;
        let louder = (sound - first_second)* 100.0 /first_second;
        println!("sound in  is #{louder} % louder than first second" );
        FreeRtos::delay_ms(100);
    }
    println!("\n End of example");
    FreeRtos::delay_ms(u32::MAX);
}

pub fn smooth_read_during(adc_driver : &mut AdcChannelDriver<'static, Gpio5, AdcDriver<'static, ADC1>>, ms: u16)-> Result<u16, EspError>{
    let mut smooth_val: u64 = 0;
    let duration = Duration::from_millis(ms as u64);
    let starting_time  = Instant::now();
    let mut amount_of_samples = 0;
    while starting_time.elapsed() < duration{
        let read_val = adc_driver.read()?;
        smooth_val += read_val as u64;
        amount_of_samples += 1;
    }
    let result = smooth_val / amount_of_samples as u64;
    Ok(result as u16)
}
