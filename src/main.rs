
/*
use std::thread;
use std::time::Duration;
use esp_idf_svc::hal::adc::config::Config;
use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals;
use esp_idf_svc::hal::peripherals::Peripherals;
use microcontroller::Microcontroller;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
*/
// mod gpio;
// mod utils;
// mod microcontroller;
// use crate::microcontroller::microcontroller::Microcontroller;
// use crate::gpio::digital_in::{InterruptType, DigitalIn};

// use esp_idf_svc::hal::delay::FreeRtos;


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


// fn main(){
//     let mut micro = Microcontroller::new();
//     println!("Configuring output channel");
    
//     let frec = 10;
//     let pin = 4; 
//     let resolution = 12;
//     let mut analog_out = micro.set_pin_as_analog_out(pin, frec * 1000, resolution);
    
//     let pin_num = 5;
//     let analog_in_pwm = micro.set_pin_as_analog_in_pwm(pin_num, frec * 1000);
    
//     println!("Starting duty-cycle loop");

//     for ratio in [0.0, 0.2, 0.4, 0.6, 0.8, 1.0].iter().cycle() {
//         println!("Duty {ratio}");
//         analog_out.set_high_level_output_ratio(*ratio as f32).unwrap();
        
//         for i in 0..3 {
//             let read_val = analog_in_pwm.read();
//             println!("Percentage sent {}, on read {}:  percentage received: {} %", ratio, i, read_val);
//         }
//         FreeRtos::delay_ms(500);
//     }

//     loop {
//         FreeRtos::delay_ms(1000);
//     }
// }

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

/*
This main intends to test the percentage of error of our implementation for
reading the PWM signals.
Different intensity levels of PWM signals will be written on a pin A, that pin
will be physically connected with pin B, and we will be reading the values received
on pin B.
All these data will be collected and then be analyzed with the intention of getting
a percentage of the error.
*/

use rand::prelude::*;
use esp32framework::{gpio::{analog_in_pwm::AnalogInPwm, analog_out::AnalogOut}, microcontroller::*};

use esp_idf_svc::hal::delay::FreeRtos;


fn main(){
    let mut micro = microcontroller::Microcontroller::new();
    
    // Different sets of configurations will be tested for the input and output

    // // Config A
    // // Out: frequency: 10 kHz | resolution: 12 bits
    // // In:  frequency: 5 kHz
    // let output_pin_b = 2; 
    // let output_freq_b = 10 * 1000;
    // let output_res_b = 12;
    // let mut analog_out_b = micro.set_pin_as_analog_out(output_pin_b, output_freq_b, output_res_b);
    // let input_pin_b = 3;
    // let input_freq_b = 5 * 1000;
    // let mut analog_in_pwm_b = micro.set_pin_as_analog_in_pwm(input_pin_b, input_freq_b);
    // analog_in_pwm_b.set_sampling(5000);

    // Config B
    // Out: frequency: 10 kHz | resolution: 12 bits
    // In:  frequency: 10 kHz
    // let output_pin_a = 4; 
    // let output_freq_a = 10 * 1000;
    // let output_res_a = 12;
    // let mut analog_out_a = micro.set_pin_as_analog_out(output_pin_a, output_freq_a, output_res_a);
    // let input_pin_a = 5;
    // let input_freq_a = 10 * 1000;
    // let mut analog_in_pwm_a = micro.set_pin_as_analog_in_pwm(input_pin_a, input_freq_a);
    // analog_in_pwm_a.set_sampling(10000);

    // Config C
    // Out: frequency: 10 kHz | resolution: 12 bits
    // In:  frequency: 20 kHz
    let output_freq_b = 10 * 1000;
    let output_res_b = 12;
    let output_pin_b = 4; 
    let mut analog_out_b = micro.set_pin_as_analog_out(output_pin_b, output_freq_b, output_res_b);
    let input_pin_b = 5;
    let input_freq_b = 20 * 1000;
    let mut analog_in_pwm_b = micro.set_pin_as_analog_in_pwm(input_pin_b, input_freq_b);
    analog_in_pwm_b.set_sampling(20000);

    
    println!("Starting duty-cycle loop");
    let mut random_generator = rand::thread_rng();
    
    println!("iteration,frequency_in,frequency_out,resolution_out,ratio,read_val");
    loop {
        let duty = random_generator.gen_range(0..1000);
        let ratio: f32 = duty as f32 / 1000 as f32;

        // Read of Config A
        //read_config(&mut analog_out_a, &analog_in_pwm_a, &ratio, &input_freq_a, &output_freq_a, &output_res_a);

        // Read of Config B
        read_config(&mut analog_out_b, &analog_in_pwm_b, &ratio, &input_freq_b, &output_freq_b, &output_res_b);

        FreeRtos::delay_ms(50);
    }
}

fn read_config(analog_out: &mut AnalogOut, analog_in_pwm: &AnalogInPwm, ratio: &f32, frequency_in: &u32, frequency_out: &u32, resolution_out: &u32) {
    analog_out.set_high_level_output_ratio(*ratio).unwrap();
    for i in 0..1 {
        let read_val = analog_in_pwm.read();
        // let line = format!("Iteration [{}] | Frequency IN [{}] | Frequency OUT [{}] | Resolution OUT [{}] | Ratio [{}] | Read Val [{}]", i, frequency_in, frequency_out, resolution_out, ratio, read_val); // Maybe i should also put the iteration number or make a mean
        let line = format!("{},{},{},{},{},{}", i, frequency_in, frequency_out, resolution_out, ratio, read_val); // Maybe i should also put the iteration number or make a mean
        println!("{}",line);
    }
}
