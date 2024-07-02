/*
mod analog_in;
mod digital_out;
mod digital_in;
mod timer_driver;
mod microcontroller;
mod peripherals;
mod error_text_parser;

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

use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;

fn main() {
    esp_idf_svc::sys::link_patches();

    println!("Configuring output channels");

    let peripherals = Peripherals::take().unwrap();

    // Configurar canales LEDC para cada color del LED RGB
    let mut red_channel = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &config::TimerConfig::new().frequency(1000.Hz().into()),  // Frecuencia para rojo (1 kHz)
        ).unwrap(),
        peripherals.pins.gpio12,  // GPIO 12 para el color rojo
    ).unwrap();

    let mut green_channel = LedcDriver::new(
        peripherals.ledc.channel1,
        LedcTimerDriver::new(
            peripherals.ledc.timer1,
            &config::TimerConfig::new().frequency(1000.Hz().into()),  // Frecuencia para verde (1 kHz)
        ).unwrap(),
        peripherals.pins.gpio13,  // GPIO 13 para el color verde
    ).unwrap();

    let mut blue_channel = LedcDriver::new(
        peripherals.ledc.channel2,
        LedcTimerDriver::new(
            peripherals.ledc.timer2,
            &config::TimerConfig::new().frequency(1000.Hz().into()),  // Frecuencia para azul (1 kHz)
        ).unwrap(),
        peripherals.pins.gpio14,  // GPIO 14 para el color azul
    ).unwrap();

    println!("Starting color change loop");

    let max_duty = red_channel.get_max_duty();

    loop {
        // Cambio de color gradual
        for duty in 0..=max_duty {
            red_channel.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5);
        }
        for duty in (0..=max_duty).rev() {
            red_channel.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5);
        }

        for duty in 0..=max_duty {
            green_channel.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5);
        }
        for duty in (0..=max_duty).rev() {
            green_channel.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5);
        }

        for duty in 0..=max_duty {
            blue_channel.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5);
        }
        for duty in (0..=max_duty).rev() {
            blue_channel.set_duty(duty).unwrap();
            FreeRtos::delay_ms(5);
        }
    }
}


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
