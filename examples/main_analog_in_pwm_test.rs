/*
This main intends to test the percentage of error of our implementation for
reading the PWM signals.
The same intensity level of PWM signals will be written on a pin A, that pin
will be physically connected with pin B, and we will be reading the values received
on pin B with different input frequencies.
All these data will be collected and then be analyzed with the intention of getting
a percentage of the error.
*/

use rand::prelude::*;
use esp32framework::{gpio::{analog_in_pwm::AnalogInPwm, analog_out::AnalogOut}, microcontroller::*};

use esp_idf_svc::hal::delay::FreeRtos;

const MAX_LOOPS: u32 = 10000;
const READS_PER_LOOP: u32 = 5; 

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
    let output_pin_a = 4; 
    let output_freq_a = 10 * 1000;
    let output_res_a = 12;
    let mut analog_out_a = micro.set_pin_as_analog_out(output_pin_a, output_freq_a, output_res_a);
    let input_pin_a = 5;
    let input_freq_a = 10 * 1000;
    let mut analog_in_pwm_a = micro.set_pin_as_analog_in_pwm(input_pin_a, input_freq_a);
    analog_in_pwm_a.set_sampling(10000);

    // Config C
    // Out: frequency: 10 kHz | resolution: 12 bits
    // In:  frequency: 20 kHz
    let output_freq_b = 10 * 1000;
    let output_res_b = 12;
    let output_pin_b = 2; 
    let mut analog_out_b = micro.set_pin_as_analog_out(output_pin_b, output_freq_b, output_res_b);
    let input_pin_b = 3;
    let input_freq_b = 20 * 1000;
    let mut analog_in_pwm_b = micro.set_pin_as_analog_in_pwm(input_pin_b, input_freq_b);
    analog_in_pwm_b.set_sampling(20000);

    // Config D
    // Out: frequency: 10 kHz | resolution: 12 bits
    // In:  frequency: 40 kHz
    // let output_freq_a = 10 * 1000;
    // let output_res_a = 12;
    // let output_pin_a = 4;
    // let mut analog_out_a = micro.set_pin_as_analog_out(output_pin_a, output_freq_a, output_res_a);
    // let input_pin_a = 5;
    // let input_freq_a = 40 * 1000;
    // let mut analog_in_pwm_a = micro.set_pin_as_analog_in_pwm(input_pin_a, input_freq_a);
    // analog_in_pwm_a.set_sampling(40000);

    // Config E
    // Out: frequency: 10 kHz | resolution: 12 bits
    // In:  frequency: 60 kHz
    // let output_freq_b = 10 * 1000;
    // let output_res_b = 12;
    // let output_pin_b = 4; 
    // let mut analog_out_b = micro.set_pin_as_analog_out(output_pin_b, output_freq_b, output_res_b);
    // let input_pin_b = 5;
    // let input_freq_b = 60 * 1000;
    // let mut analog_in_pwm_b = micro.set_pin_as_analog_in_pwm(input_pin_b, input_freq_b);
    // analog_in_pwm_b.set_sampling(60000);

    
    println!("Starting duty-cycle loop");
    let mut random_generator = rand::thread_rng();
    
    println!("iteration,frequency_in,frequency_out,resolution_out,ratio,read_val");
    for _ in 0..MAX_LOOPS {
        let duty = random_generator.gen_range(0..1000);
        let ratio: f32 = duty as f32 / 1000 as f32;

        // Read of Config A
        read_config(&mut analog_out_a, &analog_in_pwm_a, &ratio, &input_freq_a, &output_freq_a, &output_res_a);

        // Read of Config B
        read_config(&mut analog_out_b, &analog_in_pwm_b, &ratio, &input_freq_b, &output_freq_b, &output_res_b);
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}

fn read_config(analog_out: &mut AnalogOut, analog_in_pwm: &AnalogInPwm, ratio: &f32, frequency_in: &u32, frequency_out: &u32, resolution_out: &u32) {
    analog_out.set_high_level_output_ratio(*ratio).unwrap();
    FreeRtos::delay_ms(50/2);
    for i in 0..READS_PER_LOOP {
        let read_val = analog_in_pwm.read();
        let line = format!("{},{},{},{},{},{}", i, frequency_in, frequency_out, resolution_out, ratio, read_val);
        println!("{}",line);
    }
}
