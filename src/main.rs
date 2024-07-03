mod gpio;
mod utils;
mod microcontroller;
/*

use std::thread;
use std::time::Duration;

use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
//use esp_idf_svc::hal::peripherals;
use esp_idf_svc::hal::peripherals::Peripherals;
use microcontroller::Microcontroller;
*/

use microcontroller::Microcontroller;
use digital_in::{InterruptType, DigitalIn};

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

/*
fn main(){
    let mut micro = Microcontroller::new();
    println!("Configuring output channel");
    
    let frec = 10;

    // Set ledC to create a PWM signal

    let peripherals = Peripherals::take().unwrap();
    let mut channel = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &config::TimerConfig::new().frequency((100).kHz().into()).resolution(Resolution::Bits5),
        ).unwrap(),
        peripherals.pins.gpio4,
    ).unwrap();

    let digital_in = micro.set_pin_as_digital_in(5, InterruptType::PosEdge);
    
    println!("Starting duty-cycle loop");

    let max_duty = channel.get_max_duty();
    for numerator in [0, 1, 2, 3, 4, 5].iter().cycle() {
        println!("Duty {numerator}/5");
        channel.set_duty(max_duty * numerator / 5).unwrap();
        
        for i in 0..3{
            let second_method = second_read_method(frec, &digital_in, numerator);
            let first_method = first_read_method(2* frec * 1000, &digital_in, numerator);
            println!("Percentage sent {}, on read {}:  percentage 1st method: {} %   |   percentage 2nd method: {} %", numerator, i, first_method, second_method);
        }

        FreeRtos::delay_ms(500);
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}*/


fn main(){
    let mut micro = Microcontroller::new();
    println!("Configuring output channel");
    
    let frec = 10;
    let pin = 4; 
    let resolution = 12;
    let mut analog_out = micro.set_pin_as_analog_out(pin, frec * 1000, resolution);
    
    let pin_num = 5;
    let analog_in_pwm = micro.set_pin_as_analog_in_pwm(pin_num, frec * 1000);
    
    println!("Starting duty-cycle loop");

    for ratio in [0.0, 0.2, 0.4, 0.6, 0.8, 1.0].iter().cycle() {
        println!("Duty {ratio}");
        analog_out.set_high_level_output_ratio(*ratio as f32).unwrap();
        
        for i in 0..3 {
            let read_val = analog_in_pwm.read();
            println!("Percentage sent {}, on read {}:  percentage received: {} %", ratio, i, read_val);
        }
        FreeRtos::delay_ms(500);
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}

// fn second_read_method(frec: u32, digital_in: &DigitalIn)-> f32{
//     let mut reads = 0.0;
//     let amount_of_reads = 100;
//     for _i in 0..amount_of_reads{
//         reads += first_read_method(2* frec, digital_in)
//     }
//     return reads / (amount_of_reads as f32)
// }

// fn first_read_method(reading: u32, digital_in: &DigitalIn)-> f32{
//     let mut highs = 0;
//     for _num in 0..(reading){
//         if digital_in.is_high(){
//             highs += 1
//         }
//     } 
//     let a: f32 = (highs as f32) / (reading as f32);
    
//     return a
// }

