//! Blinks an LED
//!
//! This assumes that a LED is connected to GPIO4.
//! Depending on your target and the board you are using you should change the pin.
//! If your board doesn't have on-board LEDs don't forget to add an appropriate resistor.
//!
use std::sync::atomic::{AtomicBool, Ordering};
use esp_idf_svc::
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripherals::Peripherals;

static FLAG: AtomicBool = AtomicBool::new(false);

fn main() -> ! {
  esp_idf_svc::sys::link_patches();

  let dp = Peripherals::take().unwrap();

  let mut button = PinDriver::input(dp.pins.gpio0).unwrap();
  button.set_pull(Pull::Up).unwrap();

  button.set_interrupt_type(InterruptType::AnyEdge).unwrap();

  unsafe {
    button.subscribe(gpio_int_callback).unwrap();
  }

  button.enable_interrupt().unwrap();

  let mut count = 0_u32;
  
  loop {
    if FLAG.load(Ordering::Relaxed) {
      FLAG.store(false, Ordering::Relaxed);
      count = count.wrapping_add(1);

      println!("Press Count {}", count);
      FreeRtos::delay_ms(200_u32);
      button.enable_interrupt().unwrap();
    } 
  }
}

fn gpio_int_callback() {
  FLAG.store(true, Ordering::Relaxed);
}