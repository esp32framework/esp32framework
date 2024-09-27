//! Example using pin GPIO3 as analog PWM out in order to control the intensity
//! of the colours Red of a RGB led. The intensity should "bounce" when it reaches
//! the maximum and minimum level.

use std::sync::Arc;
use std::{thread,cmp,time::Duration};
use esp_idf_svc::hal::{ledc::*,peripherals::Peripherals,prelude::*};

fn duty_from_high_ratio(max_duty: u32, high_ratio: f32) -> u32{
    ((max_duty as f32) * high_ratio) as u32
}

fn get_next_red_level(led: &LedcDriver , increasing: &mut bool, change_ratio: f32) -> u32 {
    let current_duty_level = led.get_duty();
    let duty_step = duty_from_high_ratio(led.get_max_duty(), change_ratio).max(1);
    if current_duty_level >= led.get_max_duty(){
        *increasing = false;
    }else if current_duty_level == 0{
        *increasing = true;
    }
    if *increasing{
        return cmp::min(current_duty_level + duty_step ,led.get_max_duty());
    }
    
    if current_duty_level < duty_step {
        return 0;
    }
    current_duty_level - duty_step
}   

fn main() {
    esp_idf_svc::sys::link_patches();
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
        let red_level = get_next_red_level(&red_pwm, &mut increasing, 0.05);
        println!("Seteo duty en: {}", red_level);
        red_pwm.set_duty(red_level).unwrap();
        thread::sleep(Duration::from_millis(100));
    }
}
