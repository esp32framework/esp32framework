//! Blinks an LED
//!
//! This assumes that a LED is connected to GPIO4.
//! Depending on your target and the board you are using you should change the pin.
//! If your board doesn't have on-board LEDs don't forget to add an appropriate resistor.
//!
use std::borrow::Borrow;
use std::sync::atomic::{AtomicBool, Ordering};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripheral::Peripheral;
//use esp_idf_svc::hal::adc::*;
use esp_idf_svc::hal::{prelude::*, ledc::{LEDC, LedcTimer, config, LedcChannel, LedcDriver, LedcTimerDriver}};
use esp_idf_svc::hal::peripherals::Peripherals;

// mod digital;

static FLAG: AtomicBool = AtomicBool::new(false);
static LED_FLAG: AtomicBool = AtomicBool::new(false);

fn main(){
  esp_idf_svc::sys::link_patches();
  let mut peripherals = Peripherals::take().unwrap();
  button(peripherals);

}

fn led(){
    println!("Configuring output channel");

    let peripherals = Peripherals::take().unwrap();
    let mut channel = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &config::TimerConfig::new().frequency(25.kHz().into()),
        ).unwrap(),
        peripherals.pins.gpio8,
    ).unwrap();

    println!("Starting duty-cycle loop");

    let max_duty = channel.get_max_duty();
    for numerator in [0, 1, 2, 3, 4, 5].iter().cycle() {
        println!("Duty {numerator}/5");
        channel.set_duty(max_duty * numerator / 5).unwrap();
        FreeRtos::delay_ms(2000);
        channel.set_duty(max_duty / 100).unwrap(); 
        FreeRtos::delay_ms(2000);
    }

    loop {
        FreeRtos::delay_ms(1000);
    }
}


fn button(mut dp: Peripherals){
  let mut channel = LedcDriver::new(
    dp.ledc.channel0,
    LedcTimerDriver::new(
      dp.ledc.timer0,
      &config::TimerConfig::new().frequency(25.kHz().into()),
    ).unwrap(),
    &mut dp.pins.gpio8,
  ).unwrap();

  let a = dp.pins.gpio0;

  let pin: Gpio9 = dp.pins.gpio9;
  let mut a = PinDriver::output(pin).unwrap();
  a.set_high();

  let mut button = PinDriver::input(dp.pins.gpio9).unwrap();  
  button.set_pull(Pull::Up).unwrap();
  let max_duty = channel.get_max_duty();
  button.set_interrupt_type(InterruptType::AnyEdge).unwrap();
  
  unsafe {
    button.subscribe(gpio_int_callback).unwrap();
  }
  
  button.enable_interrupt().unwrap();
  
  let mut count = 0_u32;
  let mut last_state = false;
  loop {
    if FLAG.load(Ordering::Relaxed) {
      FLAG.store(false, Ordering::Relaxed);
      count = count.wrapping_add(1);
      
      if LED_FLAG.load(Ordering::Relaxed) == last_state{
        continue;
      }

      println!("Press Count {}", count);
      if LED_FLAG.load(Ordering::Relaxed){
        println!("Apago el led");
        channel.set_duty(1).unwrap();
        last_state = false;
      }else{
        println!("Prendo el led");
        channel.set_duty(max_duty).unwrap();
        last_state = true;
      }
      FreeRtos::delay_ms(200_u32);
      button.enable_interrupt().unwrap();
    } 
    FreeRtos::delay_ms(200_u32);
  }
}

fn gpio_int_callback() {
  FLAG.store(true, Ordering::Relaxed);
  if LED_FLAG.load(Ordering::Relaxed){
    LED_FLAG.store(false, Ordering::Relaxed);
  }else{
    LED_FLAG.store(true, Ordering::Relaxed);
  }
}

/*
*/


/*
#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_println::println;
use esp_hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("Hello world!");

    // Set GPIO7 as an output, and set its state high initially.
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio7.into_push_pull_output();

    led.set_high().unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    loop {
        led.toggle().unwrap();
        delay.delay_ms(500u32);
    }
}
*/