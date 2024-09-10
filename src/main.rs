// use std::sync::{atomic::AtomicU8, Arc};

use esp32framework::{ble::{ble_client::BleClient, BleId}, Microcontroller};
use esp32_nimble::BLEDevice;
fn main(){
  let mut micro = Microcontroller::new();
  
  let ble_device = BLEDevice::take();
  let mut client = BleClient::new(ble_device);
  let service_id = BleId::FromUuid32(0x2AB4);
  client.connect_to_device_of_name(None, String::from("kk")).unwrap();
  
  println!("Connected");
  micro.wait_for_updates(Some(2000));

  let services = client.get_all_services().unwrap();
  for service in &services{
    println!("Service: {:?}", service.to_uuid());
  }


  let mut characteristics = client.get_all_characteristics(&service_id).unwrap();
  for characteristic in &characteristics{
    println!("Characteristic: {:?}", characteristic.id());
  } 

  let descriptors = characteristics[0].get_all_descriptors().unwrap();
  for desc in descriptors.iter(){
    println!("Descriptor: {:?}", desc.uuid());
  }

  let descriptor_id = BleId::from(&descriptors[0].uuid());
  let descriptor_1 = characteristics[0].get_descriptor_value(&descriptor_id);
  println!("Descriptor value: {:?}", descriptor_1);

}

  // let multiplier = Arc::new(AtomicU8::new(2));
  // for characteristic in &mut characteristics{
  //   let cloned_multiplier = multiplier.clone();
  //   _ = characteristic.on_notify(move |data| {
  //     cloned_multiplier.store(data[0], std::sync::atomic::Ordering::SeqCst)
  //   });
  // }

  //TODO blockon en micro para hacer lo del usuario + los updates
  
  // recibe el future del usuario y por adentro tambien se le pasa el update: 
  
  
//   loop{
//     for characteristic in characteristics.iter_mut(){
//       let read = match characteristic.read() {
//         Ok(read) => get_number_from_bytes(read),
//         Err(err) => match err{
//           BleError::CharacteristicIsNotReadable => continue,
//           _ => Err(err).unwrap()
//         }
//       };
      
//       let mult = multiplier.load(std::sync::atomic::Ordering::Acquire);
//       let new_value = read * mult as u32;

//       println!("Read value: {}, multipling by: {}, result: {}", read, mult, new_value);
    
//       if let Err(err) = characteristic.write(&new_value.to_be_bytes()){
//         match err{
//           BleError::CharacteristicIsNotWritable => continue,
//           _ => Err(err).unwrap()
//         }
//       }
//     }
    
// 		micro.wait_for_updates(Some(2000));
// 	}
// }

// fn get_number_from_bytes(bytes: Vec<u8>)->u32{
//   let mut aux = vec![0,0,0,0];
//   aux.extend(bytes);
//   let bytes = aux.last_chunk().unwrap();
//   u32::from_be_bytes(*bytes)
// }

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
