use std::sync::Arc;
use std::thread;
use std::time::Duration;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::ledc::*;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::hal::prelude::*;
use esp_idf_sys as _;

fn get_next_red_level(led: LedcDriver , increasing: &mut bool, change_ratio: f32) -> u32{
    let current_duty_level = led.get_duty();
    if current_duty_level >= led.get_max_duty(){
        *increasing = false;
    }else if current_duty_level <= 0{
        *increasing = true;
    }
    if *increasing{
        min(current_duty_level + change_ratio,led.get_max_duty())
    }
    max(current_duty_level - change_ratio, 0)
}   

/// Example using pin GPIO3 as analog PWM out in order to control the intensity
/// of the colours Red of a RGB led. The intensity should "bounce" when it reaches
/// the maximum and minimum level.
fn main() {
    esp_idf_sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let config = config::TimerConfig::new().frequency(1.kHz().into());
    let timer = Arc::new(LedcTimerDriver::new(peripherals.ledc.timer0, &config).unwrap());
    let mut increasing: bool = true;
    
    let mut red_pwm = LedcDriver::new(
        peripherals.ledc.channel0,
        timer.clone(),
        peripherals.pins.gpio3,
    )
    .unwrap();

    loop {
        let red_level = get_next_red_level(red_pwm, &mut increasing, 0.05);
        red_pwm.set_duty(red_level).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}