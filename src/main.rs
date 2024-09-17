//! Example of a ble client using an async aproach. The client will connect to a server that has a 
//! characteristic of uuid 0x12345678. Once connected the client will read all characteristics interpreting
//! their value as an u32 and then multiplies them by a value. This value is obtained from the notifiable 
//! characteristics of the service. Thanks to the async aproch we can have other tasks running concurrently
//! to this main function. In this case there is a TimerDriver se to print 'Tic' every 2 seconds.

use std::{future::Future, sync::mpsc::{self, Receiver}, time::Duration};
use futures::{future::join};

use esp32_nimble::BLEDevice;
use esp32framework::{ble::{ble_client::BleClient, BleError, BleId, RemoteCharacteristic}, timer_driver::{self, TimerDriver}, utils::notification::Notifier, Microcontroller};
use esp_idf_svc::hal::{delay::FreeRtos, task::block_on};

fn main(){
  let mut micro = Microcontroller::new();
  let client = micro.ble_client();
  let mut timer_driver = micro.get_timer_driver();
  let mut timer_driver = micro.get_timer_driver();
  let mut button = micro.set_pin_as_digital_in(9);

  button.trigger_on_interrupt(|| {
    println!("Apreto");
  }, esp32framework::gpio::InterruptType::NegEdge).unwrap();
  
  println!("hola");
  let fut1 = connect_framework(client);
  //let fut1 = connect();
  let fut2 = print("aca");
  //let fut2 = timer_driver_sleep(timer_driver, 1);
  //let joined = join(fut1, fut2);
  micro.block_on(join(fut1, fut2));
  //micro.block_on2(fut1, fut2);

  println!("chau");
  micro.wait_for_updates(None);

  /*
  let mut characteristics = get_characteristics(&mut micro);
  
  let receiver = set_notify_callback_for_characteristics(&mut characteristics);
  let timer_driver1 = set_periodical_timer_driver_interrupts(&mut micro, 2000);
  let timer_driver3 = micro.get_timer_driver();
  let fut1 = join!(main_loop(timer_driver1, characteristics, receiver), timer_driver_sleep(timer_driver3, 1), timer_driver_sleep(timer_driver2, 2));
  //let fut3 = join!(fut2, timer_driver_sleep(timer_driver4, 3));
  micro.block_on(fut1);
  */
}

async fn print(s: &str){
  println!("{}", s)
}

async  fn a(noifier: Notifier){
  println!("Durmiendo");
  FreeRtos::delay_ms(2000);
  noifier.notify().unwrap();
}

async fn connect_framework(mut client: BleClient){
  let service_id = BleId::FromUuid32(0x12345678);
  println!("Attempting connection");
  //client.connect_to_device_with_service_async(None, &service_id).await.unwrap();
  client.connect_to_device_async(None, |_| {false}).await.unwrap();
  
  println!("Connected");
}

async fn timer_driver_sleep<'a>(mut timer_driver: TimerDriver<'a>, i: u8){
  timer_driver.delay(5000).await.unwrap();
  println!("delay {}", i)
}

async fn connect(){
  let ble_device = BLEDevice::take();
  let ble_scan = ble_device.get_scan();
  ble_scan.active_scan(true)
    .interval(100)
    .window(99);
  
  let device = ble_scan.find_device(i32::MAX, |adv| {
      false
  }).await.unwrap();
}