use esp32framework::{Microcontroller, ble::{ble_client::BleClient, BleId}};
use esp32_nimble::BLEDevice;
fn main(){
  let mut micro = Microcontroller::new();
  
  let ble_device = BLEDevice::take();
  let mut client = BleClient::new(ble_device);
  client.connect_to_device_with_service(None, &BleId::FromUuid32(0x12345678)).unwrap();
  
  println!("Connected");
  micro.wait_for_updates(Some(2000));

  let mut characteristic = client.get_characteristic(BleId::FromUuid32(0x12345678), BleId::FromUuid16(0x0101)).unwrap();
  loop{
		println!("La characteristica vale: {:?}", characteristic.read().unwrap());
		micro.wait_for_updates(Some(2000));
	}
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
