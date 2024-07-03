mod analog_in;
mod digital_out;
mod digital_in;
mod timer_driver;
mod microcontroller;
mod peripherals;
mod error_text_parser;
mod analog_in_pwm;
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
pub unsafe fn new() -> Self {
            Self {
                timer0: TIMER0::new(),
                timer1: TIMER1::new(),
                timer2: TIMER2::new(),
                timer3: TIMER3::new(),
                channel0: CHANNEL0::new(),
                channel1: CHANNEL1::new(),
                channel2: CHANNEL2::new(),
                channel3: CHANNEL3::new(),
                channel4: CHANNEL4::new(),
                channel5: CHANNEL5::new(),
                #[cfg(any(esp32, esp32s2, esp32s3, esp8684))]
                channel6: CHANNEL6::new(),
                #[cfg(any(esp32, esp32s2, esp32s3, esp8684))]
                channel7: CHANNEL7::new(),
            }
        }
*/

fn main(){
    let mut micro = Microcontroller::new();
    println!("Configuring output channel");
    
    let frec = 10;

    //Set ledC to create a PWM signal
    let peripherals = Peripherals::take().unwrap();
    let mut channel = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &config::TimerConfig::new().frequency((frec *10).kHz().into()),
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
}

fn second_read_method(frec: u32, digital_in: &DigitalIn, numerator: &u32)-> f32{
    let mut reads = 0.0;
    let amount_of_reads = 100;
    for _i in 0..amount_of_reads{
        reads += first_read_method(2* frec, digital_in, numerator)
    }
    return reads / (amount_of_reads as f32)
}


fn first_read_method(reading: u32, digital_in: &DigitalIn, numerator: &u32)-> f32{
    let mut highs = 0;
    for _num in 0..(reading){
        if digital_in.is_high(){
            highs += 1
        }
    } 
    let a: f32 = (highs as f32) / (reading as f32);
    
    
    return a
}

/*
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;


fn main() {
    // Inicializa el ESP-IDF
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::sys::pre_init();

    // Configuración inicial para MCPWM
    let mcpwm_unit = esp_idf_svc::sys::mcpwm_unit_t_MCPWM_UNIT_0;
    let mcpwm_timer = esp_idf_svc::sys::mcpwm_timer_t_MCPWM_TIMER_0;

    // Configura el pin de salida
    let gpio_num = esp_idf_svc::sys::gpio_num_t_GPIO_NUM_18;
    unsafe {
        esp!(sys::mcpwm_gpio_init(mcpwm_unit, sys::mcpwm_io_signals_t_MCPWM0A, gpio_num)).unwrap();
    }

    // Configura MCPWM
    let mut pwm_config = sys::mcpwm_config_t {
        frequency: 1000, // Frecuencia en Hz
        cmpr_a: 0.0,     // Ciclo de trabajo inicial para PWM A
        cmpr_b: 0.0,     // Ciclo de trabajo inicial para PWM B
        counter_mode: sys::mcpwm_counter_type_t_MCPWM_UP_COUNTER,
        duty_mode: sys::mcpwm_duty_type_t_MCPWM_DUTY_MODE_0,
    };

    unsafe {
        esp!(sys::mcpwm_init(mcpwm_unit, mcpwm_timer, &mut pwm_config)).unwrap();
    }

    // Establece el ciclo de trabajo para el PWM A
    let duty_cycle = 50.0; // Ciclo de trabajo en porcentaje
    unsafe {
        esp!(sys::mcpwm_set_duty(mcpwm_unit, mcpwm_timer, sys::mcpwm_generator_t_MCPWM_GEN_A, duty_cycle)).unwrap();
        esp!(sys::mcpwm_set_duty_type(mcpwm_unit, mcpwm_timer, sys::mcpwm_generator_t_MCPWM_GEN_A, sys::mcpwm_duty_type_t_MCPWM_DUTY_MODE_0)).unwrap();
    }

    loop {
        // Aquí puedes agregar la lógica para ajustar el ciclo de trabajo o realizar otras operaciones
    }
}

*/

/* output
Starting duty-cycle loop
Duty 0/5
Duty 1/5
Duty 2/5
Duty 3/5
Duty 4/5
Duty 5/5
Duty 0/5
Duty 1/5
Duty 2/5
Duty 3/5
Duty 4/5
Duty 5/5
Duty 0/5
Duty 1/5
Duty 2/5
Duty 3/5
 */
