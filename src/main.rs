//! ADC example, reading a value form a pin and printing it on the terminal
//!

//use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

mod analog_in;
mod digital_out;
mod digital_in;
mod timer_driver;
mod microcontroller;
mod peripherals;

use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::gpio::*;
//use esp_idf_svc::hal::peripherals;
use esp_idf_svc::hal::peripherals::Peripherals;


// fn a<T: IOPin>(adc_pin: T)-> T{
//     return adc_pin
// }

// struct OurPeripheral {
//     gpio0: Option<Gpio0>,
//     gpio1: Option<Gpio1>
// }

// impl OurPeripheral{
//     fn new(){
//         let peripherals = Peripherals::take().unwrap(); 
//         OurPeripheral{
//             gpio0: Some(peripherals.pins.gpio0),
//             gpio1: Some(peripherals.pins.gpio1),
//         }
//     }

// }

// fn micro(self){
//     let a: Gpio0: self.peripherals.gpio0.take();
// }

// impl OurPin for esp_idf_svc::hal::gpio::Gpio0{
    
// }

// impl OurPin for Gpio1{
// }


// fn perf<O: IOPin>(a: bool)-> impl AN{
//     let peripherals = Peripherals::take().unwrap();
//     peripherals.i2c0
//     if a{
//         return peripherals.pins.gpio19
//     }
//     return peripherals.pins.gpio0

// }

fn main() {
    let peripherals = Peripherals::take().unwrap();
    //let mut adc = AdcDriver::new(peripherals.adc1, &Config::new().calibration(true)).unwrap();
    let mut pin = peripherals.pins.gpio6;
    let mut pin2 = peripherals.pins.gpio7;
    let pin3 = peripherals.i2c0;
    let pi43 = peripherals.i2s0;
    // let adc_pin= a(pin);
    
    loop {
        println!("No exploto");
        thread::sleep(Duration::from_millis(200));
    }

    // configuring pin to analog read, you can regulate the adc input voltage range depending on your need
    // for this example we use the attenuation of 11db which sets the input voltage range to around 0-3.6V
    // let mut adc_pin: esp_idf_svc::hal::adc::AdcChannelDriver<{ attenuation::DB_11 }, _> =
    //     AdcChannelDriver::new(adc_pin).unwrap();
    // loop {
    //     thread::sleep(Duration::from_millis(10));
    //     println!("ADC value: {}", adc.read(&mut adc_pin).unwrap());
    // }
}