// use std::sync::{atomic::AtomicU8, Arc};

use std::{borrow::BorrowMut, sync::{atomic::AtomicU8, mpsc::{self, Receiver}, Arc}};

use esp32framework::{ble::{ble_client::BleClient, BleError, BleId, Characteristic, RemoteCharacteristic}, timer_driver::TimerDriver, Microcontroller};
use esp32_nimble::BLEDevice;

fn main(){
  let mut micro = Microcontroller::new();

  let mut characteristics = get_characteristics(&mut micro);

  let receiver = set_notify_callback_for_characteristics(&mut micro, &mut characteristics);
  let timer_driver = set_periodical_timer_driver_interrupts(&mut micro, 2000);

  micro.block_on(main_loop(timer_driver, characteristics, receiver))
}

fn get_characteristics(micro: &mut Microcontroller)-> Vec<RemoteCharacteristic>{
  let mut client = micro.ble_client();
  let service_id = BleId::FromUuid32(0x12345678);
  client.connect_to_device_with_service(None, &service_id).unwrap();
  
  println!("Connected");
  micro.wait_for_updates(Some(2000));
  
  client.get_all_characteristics(&service_id).unwrap()
  
}

fn set_notify_callback_for_characteristics(micro: &mut Microcontroller, characteristics: &mut Vec<RemoteCharacteristic>)-> Receiver<u8>{
  let (sender, receiver) = mpsc::channel();

  for characteristic in characteristics{
    let s = sender.clone();
    _ = characteristic.on_notify(move |data| {
      s.send(data[0]);
    });
  }

  receiver
}

fn set_periodical_timer_driver_interrupts<'a>(micro: &mut Microcontroller<'a>, mili: u64)-> TimerDriver<'a>{
  let mut timer_driver = micro.get_timer_driver();
  timer_driver.interrupt_after_n_times(mili * 1000, None, true, || {println!("INTERRUPCION")});
  timer_driver.enable().unwrap();
  timer_driver
}

async fn main_loop<'a>(mut timer_driver: TimerDriver<'a>,mut characteristics: Vec<RemoteCharacteristic>, receiver: Receiver<u8>){
  let mut mult = 2;
  loop{
    for characteristic in characteristics.iter_mut(){
      let read = match characteristic.read_async().await {
        Ok(read) => get_number_from_bytes(read),
        Err(err) => match err{
          BleError::CharacteristicIsNotReadable => continue,
          _ => Err(err).unwrap()
        }
      };
      
      if let Ok(new_mult) = receiver.try_recv(){
        mult = new_mult
      }
      let new_value = read.wrapping_mul(mult as u32);

      println!("Read value: {}, multipling by: {}, result: {}", read, mult, new_value);
    
      if let Err(err) = characteristic.write_async(&new_value.to_be_bytes()).await{
        match err{
          BleError::CharacteristicIsNotWritable => continue,
          _ => Err(err).unwrap()
        }
      }
    }

    timer_driver.delay(4000).await.unwrap();
	}
}


fn get_number_from_bytes(bytes: Vec<u8>)->u32{
  let mut aux = vec![0,0,0,0];
  aux.extend(bytes);
  let bytes = aux.last_chunk().unwrap();
  u32::from_be_bytes(*bytes)
}
/*
use bstr::ByteSlice;
use esp32_nimble::{utilities::BleUuid, uuid128, BLEClient, BLEDevice};
use esp_idf_svc::hal::{
  delay::FreeRtos, prelude::Peripherals, task::block_on, timer::{TimerConfig, TimerDriver}
};

fn main() {
  esp_idf_svc::sys::link_patches();
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut timer = TimerDriver::new(peripherals.timer00, &TimerConfig::new()).unwrap();

  block_on(async {
    let ble_device = BLEDevice::take();
    let ble_scan = ble_device.get_scan();
    let device = ble_scan
      .active_scan(true)
      .interval(100)
      .window(99)
      .find_device(10000, |device| {
          println!("device_name: {}", device.name());
          device.name().contains_str("e")
        })
      .await.unwrap();

    if let Some(device) = device {
      let mut client = BLEClient::new();
      client.on_connect(|client| {
        client.update_conn_params(120, 120, 0, 60).unwrap();
      });
      client.connect(device.addr()).await.unwrap();

      for s in client.get_services().await.unwrap(){
        println!("service: {s}");
      }
      
      let service = client
        .get_service(BleUuid::Uuid16(0x1515))
        .await.unwrap();

      let uuid = BleUuid::Uuid16(0x0101);
      let characteristic = service.get_characteristic(uuid).await.unwrap();
      let value = characteristic.read_value().await.unwrap();
      println!("{characteristic} tiene value: {:?}", value);
      
      let uuid = BleUuid::Uuid16(0x0202);
      let characteristic = service.get_characteristic(uuid).await.unwrap();

      if !characteristic.can_notify() {
        ::log::error!("characteristic can't notify: {}", characteristic);
      }

      println!("subscribe to {}", characteristic);
      characteristic
        .on_notify(move |data| {
        println!("{uuid} notified: {:?}", data)
        })
        .subscribe_notify(false)
        .await.unwrap();
      timer.delay(timer.tick_hz() * 50).await.unwrap();

      client.disconnect().unwrap();
    }
  });
  FreeRtos::delay_ms(5000)
}

 */
