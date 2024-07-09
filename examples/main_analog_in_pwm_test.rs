/*
This main intends to test the percentage of error of our implementation for
reading the PWM signals.
Different intensity levels of PWM signals will be written on a pin A, that pin
will be physically connected with pin B, and we will be reading the values received
on pin B.
All these data will be collected and then be analyzed with the intention of getting
a percentage of the error.
*/

use esp32framework::microcontroller::*;

use esp_idf_svc::hal::delay::FreeRtos;
use std::fs::File;

const OUTPUT_FILE: str = "output.txt";

fn main(){
    let mut micro = microcontroller::Microcontroller::new();
    
    let file = File::create(OUTPUT_FILE).unwrap();
    
    // Different sets of configurations will be tested for the input and output
    
    // Config A: 
    // Out: frequency: 10 kHz | resolution: 12 bits
    // In:  frequency: 10 kHz
    let output_pin_a = 4; 
    let output_freq_a = 10 * 1000;
    let output_res_a = 12;
    let mut analog_out_a = micro.set_pin_as_analog_out(pin, output_freq, output_res);
    let input_pin_a = 5;
    let input_freq_a = 10 * 1000;
    let analog_in_pwm_a = micro.set_pin_as_analog_in_pwm(input_pin, freq_a);

    // Config B
    // Out: frequency: 20 kHz | resolution: 12 bits
    // In:  frequency: 10 kHz
    let output_pin_b = 2; 
    let output_freq_b = 20 * 1000;
    let output_res_b = 12;
    let mut analog_out_b = micro.set_pin_as_analog_out(pin, output_freq, output_res);
    let input_pin_b = 3;
    let input_freq_b = 10 * 1000;
    let analog_in_pwm_b = micro.set_pin_as_analog_in_pwm(input_pin, freq_a);

    // Config C
    // Out: frequency: 20 kHz | resolution: 12 bits
    // In:  frequency: 5 kHz
    // let output_pin_c = 0; 
    // let output_freq_c = 20 * 1000;
    // let output_res_c = 12;
    // let mut analog_out_c = micro.set_pin_as_analog_out(pin, output_freq, output_res);
    // let input_pin_c = 1;
    // let input_freq_c = 5 * 1000;
    // let analog_in_pwm_c = micro.set_pin_as_analog_in_pwm(input_pin, freq_a);

    // Config D
    // Out: frequency: 7 kHz | resolution: 12 bits
    // In:  frequency: 14 kHz
    // let output_freq_d = 7 * 1000;
    // let output_res_d = 12;
    // let output_pin_d = ??; 
    // let mut analog_out_d = micro.set_pin_as_analog_out(pin, output_freq, output_res);
    // let input_pin_d = ??;
    // let input_freq_d = 14 * 1000;
    // let analog_in_pwm_d = micro.set_pin_as_analog_in_pwm(input_pin, freq_a);

    
    println!("Starting duty-cycle loop");

    for ratio in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0].iter().cycle() {
        // Read of Config A
        read_config(analog_out_a, analog_in_pwm_a, duty, "A");
        // Read of Config B
        read_config(analog_out_b, analog_in_pwm_b, duty, "B");
        // Read of COnfig C
        read_config(analog_out_c, analog_in_pwm_c, duty, "C");

        FreeRtos::delay_ms(500);
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}

fn read_config(analog_out: AnalogOut, analog_in_pwm: Analog_in, duty: f64, config: str) {
    analog_out.set_high_level_output_ratio(*ratio as f32).unwrap();
    for i in 0..3 {
        let read_val = analog_in_pwm.read();
        let line = format!("Config [{}] with duty [{}]: [{}]", config, duty, read_val); // Maybe i should also put the iteration number or make a mean
        println!(line)
    }
}