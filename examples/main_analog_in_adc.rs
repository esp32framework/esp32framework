//! ADC example, reading a value form a pin and printing it on the terminal
//!

//use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

// mod analog_in;
// mod digital_out;
// mod digital_in;
// mod timer_driver;
// mod microcontroller;
// mod peripherals;
// mod error_text_parser;
use esp32framework::Microcontroller;
//use std::thread;
//use std::time::Duration;

// use esp_idf_svc::hal::adc::config::Config;
//use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::delay::FreeRtos;
//use esp_idf_svc::hal::gpio::*;
//use esp_idf_svc::hal::peripherals;
//use esp_idf_svc::hal::peripherals::Peripherals;
// use microcontroller::Microcontroller;

/// Main for analog in with esp hal
// fn main() {
//     let peripherals = Peripherals::take().unwrap();
//     let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
//     let mut pin = peripherals.pins.gpio6;
//     // let mut pin2 = peripherals.pins.gpio7;
//     // let pin3 = peripherals.i2c0;
//     // let pi43 = peripherals.i2s0;
//     // let adc_pin= a(pin);
//     // configuring pin to analog read, you can regulate the adc input voltage range depending on your need
//     // for this example we use the attenuation of 11db which sets the input voltage range to around 0-3.6V
//     let mut adc_pin: esp_idf_svc::hal::adc::AdcChannelDriver<{ attenuation::DB_11 }, _> =
//         AdcChannelDriver::new(pin).unwrap();
//     loop {
//         thread::sleep(Duration::from_millis(10));
//         println!("ADC value: {}", adc.read(&mut adc_pin).unwrap());
//     }
// }

///  Main for our analog
fn main(){
    let mut micro = Microcontroller::new();
    let mut analog_in = micro.set_pin_as_analog_in_low_atten(0);
    loop {
        let read = analog_in.read().unwrap();
        let raw_read = analog_in.read_raw().unwrap();
        let smooth_read = analog_in.smooth_read(20).unwrap();
        println!("READ: {read} | RAW: {raw_read} | SMOOTH: {smooth_read}");
        FreeRtos::delay_ms(500_u32);
        micro.update();
    }
    //drop(analog_in);
    //println!("{:?}", micro);
    //drop(micro);
}


