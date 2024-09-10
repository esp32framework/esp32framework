use std::sync::{atomic::AtomicU8, Arc};

use esp32framework::{ble::{ble_client::BleClient, BleError, BleId, RemoteCharacteristic}, Microcontroller};
use esp32_nimble::BLEDevice;
fn main(){
  let mut micro = Microcontroller::new();
  
  let ble_device = BLEDevice::take();
  let mut client = BleClient::new(ble_device);
  let service_id = BleId::FromUuid32(0x12345678);
  client.connect_to_device_with_service(None, &service_id).unwrap();
  
  println!("Connected");
  micro.wait_for_updates(Some(2000));

  let mut characteristics = client.get_all_characteristics(&service_id).unwrap();
  let multiplier = Arc::new(AtomicU8::new(2));
  for characteristic in &mut characteristics{
    let cloned_multiplier = multiplier.clone();
    _ = characteristic.on_notify(move |data| {
      cloned_multiplier.store(data[0], std::sync::atomic::Ordering::SeqCst)
    });
  }

  let mut timer_driver = micro.get_timer_driver();
  timer_driver.interrupt_after(2000, || {println!("INTERRUPCION")});
  timer_driver.enable().unwrap();

  micro.block_on(main_loop( characteristics, multiplier))
  

  //TODO blockon en micro para hacer lo del usuario + los updates
  
  // recibe el future del usuario y por adentro tambien se le pasa el update: 
  
  
}

async fn main_loop<'a>(mut characteristics: Vec<RemoteCharacteristic<'a>>, multiplier: Arc<AtomicU8>){
  loop{
    for characteristic in characteristics.iter_mut(){
      let read = match characteristic.read_async().await {
        Ok(read) => get_number_from_bytes(read),
        Err(err) => match err{
          BleError::CharacteristicIsNotReadable => continue,
          _ => Err(err).unwrap()
        }
      };
      
      let mult = multiplier.load(std::sync::atomic::Ordering::Acquire);
      let new_value = read.wrapping_mul(mult as u32);

      //println!("Read value: {}, multipling by: {}, result: {}", read, mult, new_value);
    
      if let Err(err) = characteristic.write_async(&new_value.to_be_bytes()).await{
        match err{
          BleError::CharacteristicIsNotWritable => continue,
          _ => Err(err).unwrap()
        }
      }
    }
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
