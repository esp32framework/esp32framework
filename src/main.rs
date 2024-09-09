use esp32framework::{Microcontroller, ble::{ble_client::BleClient, BleId}};
use esp32_nimble::{BLEAdvertisementData, BLEDevice, DescriptorProperties, NimbleProperties};
// fn main(){
//     let mut micro = Microcontroller::new();
	
//     let ble_device = BLEDevice::take();
//     let mut client = BleClient::new(ble_device);
//     client.connect_to_device_of_name(None, "Galaxy S10e".to_string()).unwrap();
	
//     println!("Connected");
//     micro.wait_for_updates(Some(2000));
//     client.prueba();
//     micro.wait_for_updates(None)
// }


use bstr::ByteSlice;
use esp32_nimble::{utilities::BleUuid, uuid128, BLEClient};
use esp_idf_svc::hal::{
  delay::FreeRtos, prelude::Peripherals, task::block_on, timer::{TimerConfig, TimerDriver}
};

fn main() {
	esp_idf_svc::sys::link_patches();
	esp_idf_svc::log::EspLogger::initialize_default();
  
	let device = BLEDevice::take();
	let ble_advertising = device.get_advertising();
  
	let server = device.get_server();
	server.on_connect(|server, desc| {
	  ::log::info!("Client connected: {:?}", desc);
  
		if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
			::log::info!("Multi-connect support: start advertising");
			ble_advertising.lock().start().unwrap();
		}
	});
	server.on_disconnect(|_desc, reason| {
	  ::log::info!("Client disconnected ({:?})", reason);
	  if let Err(e) = reason {
		  println!("El error fue {:?}", e.to_string());
	  }
	});
  
	let service = server.create_service(BleUuid::Uuid16(0xABCD));
  
	let characteristic = service
	  .lock()
	  .create_characteristic(BleUuid::Uuid16(0xAAAA), NimbleProperties::READ);
	characteristic
	  .lock()
	  .set_value("non_secure_characteristic".as_bytes());

	let desc = characteristic.lock().create_descriptor(BleUuid::Uuid16(0x11), DescriptorProperties::READ);
	desc.lock().set_value(&[0x12;1]);
  
	// With esp32-c3, advertising stops when a device is bonded.
	// (https://github.com/taks/esp32-nimble/issues/70)
	ble_advertising.lock().set_data(
	  BLEAdvertisementData::new()
		.name("ESP32-GATT-Server")
		.add_service_uuid(BleUuid::Uuid16(0xABCD)),
	).unwrap();
	ble_advertising.lock().start().unwrap();
  
	::log::info!("bonded_addresses: {:?}", device.bonded_addresses());
  
	loop {
	  esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
	}
}


