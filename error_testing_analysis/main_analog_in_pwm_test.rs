//! This main intends to test the percentage of error of our implementation for
//! reading the PWM signals.
//! The same intensity level of PWM signals will be written on a pin A, that pin
//! will be physically connected with pin B, and we will be reading the values received
//! on pin B with different input frequencies.
//! All these data will be collected and then be analyzed with the intention of getting
//! a percentage of the error.
//! 
//! Note: This file was used to test, after the testing was done libraries used where 
//! taken off the Cargo.toml. To replicate this test, the rand library should be added 
//! to the Cargo.toml.

use rand::prelude::*;
use esp32framework::{gpio::{AnalogInPwm, AnalogOut}, Microcontroller};

const MAX_LOOPS: u32 = 10000;
const READS_PER_LOOP: u32 = 5;
const FREQUENCY_OUT: u32 = 10000;
const RESOLUTION_OUT: u32 = 12;

const OUTPUT_PIN_1: usize = 2;
const INPUT_PIN_1: usize = 3;

const OUTPUT_PIN_2: usize = 4;
const INPUT_PIN_2: usize = 5;

const SLEEP_TIME: u32 = 50;


fn main(){
    let mut micro = Microcontroller::new();
    
    // Different sets of configurations will be tested for the input and output

    // Config A
    // In:  frequency: 5 kHz
    let frequency_in_a = 5 * 1000;

    // Config B
    // In:  frequency: 10 kHz
    let frequency_in_b = 10 * 1000;

    // Config C
    // In:  frequency: 20 kHz
    let frequency_in_c = 20 * 1000;

    // Config D
    // In:  frequency: 40 kHz
    let frequency_in_d = 40 * 1000;
    
    println!("Starting duty-cycle loop");
    let mut random_generator = rand::thread_rng();
    
    println!("iteration,frequency_in,frequency_out,resolution_out,ratio,read_val");
    for _ in 0..MAX_LOOPS {
        let duty = random_generator.gen_range(0..1000);
        let ratio: f32 = duty as f32 / 1000 as f32;

        // Read of Config 1
        read_config(&mut analog_out_a, &analog_in_pwm_a, &ratio, &input_freq_a, &output_freq_a, &output_res_a);

        // Read of Config 2
        read_config(&mut analog_out_b, &analog_in_pwm_b, &ratio, &input_freq_b, &output_freq_b, &output_res_b);

        micro.sleep(SLEEP_TIME);
    }

    micro.wait_for_updates(None)
}

/// Returns analog_in_1, analog_in_2, analog_out_1, analog_out_2
fn create_analogs(frequency_in_1: u32, frequency_in_2: u32, mut micro: Microcontroller) -> (AnalogInPwm, AnalogInPwm, AnalogOut, AnalogOut) {
    let mut analog_in_1 = micro.set_pin_as_analog_in_pwm(INPUT_PIN_1, frequency_in_1).unwrap();
    analog_in_1.set_sampling(frequency_in_1);
    let analog_out_1 = micro.set_pin_as_analog_out(OUTPUT_PIN_1, FREQUENCY_OUT, RESOLUTION_OUT).unwrap();
    
    let mut analog_in_2 = micro.set_pin_as_analog_in_pwm(INPUT_PIN_2, frequency_in_2).unwrap();
    analog_in_2.set_sampling(frequency_in_2);
    let analog_out_2 = micro.set_pin_as_analog_out(OUTPUT_PIN_2, FREQUENCY_OUT, RESOLUTION_OUT).unwrap();

    (analog_in_1, analog_in_2, analog_out_1, analog_out_2)

}

fn read_config(analog_out: &mut AnalogOut, analog_in_pwm: &AnalogInPwm, ratio: &f32, frequency_in: &u32, frequency_out: &u32, resolution_out: &u32) {
    analog_out.set_high_level_output_ratio(*ratio).unwrap();
    for i in 0..READS_PER_LOOP {
        let read_val = analog_in_pwm.read();
        let line = format!("{},{},{},{},{},{}", i, frequency_in, frequency_out, resolution_out, ratio, read_val);
        println!("{}",line);
    }
}
