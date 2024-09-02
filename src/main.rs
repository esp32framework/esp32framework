use esp32_nimble::{enums::*, utilities::BleUuid, BLEAdvertisementData, BLEDevice, NimbleProperties};

use esp32framework::{ble::{BleBeacon, BleId, Characteristic, IOCapabilities, Security, Service, StandarCharacteristicId, StandarServiceId}, gpio::DigitalOut, sensors::{DateTime, DS3231}, Microcontroller};

const LED: usize = 15;
const SDA: usize = 5;
const SCL: usize = 6;

fn main(){
	let mut micro = Microcontroller::new();
	let security = Security::new(123456, IOCapabilities::DisplayOnly);
	let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
	let char_id = BleId::StandarCharacteristic(StandarCharacteristicId::ActivityGoal);
	let mut characteristic: Characteristic = Characteristic::new(char_id, vec![]);
	characteristic.readeable(true).readeable_authen(true).readeable_enc(true);
	let mut service = Service::new(&service_id, vec![]).unwrap();
	service.add_characteristic(characteristic);
	let mut services = vec![service];
	let mut server = micro.ble_secure_server("Server".to_string(),&services,security);
	server.start();
	loop {
		micro.wait_for_updates(Some(300));
	}
}



/*
  
  fn main() {
	esp_idf_svc::sys::link_patches();
	esp_idf_svc::log::EspLogger::initialize_default();
  
	let device = BLEDevice::take();
	let ble_advertising = device.get_advertising();
  
	device
	  .security()
	  .set_auth(AuthReq::all())
	  .set_passkey(123456)
	  .set_io_cap(SecurityIOCap::KeyboardDisplay)
	  .resolve_rpa();
  
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
	server.on_authentication_complete(|desc, result| {
	  ::log::info!("AuthenticationComplete({:?}): {:?}", result, desc);
	});
  
	let service = server.create_service(BleUuid::Uuid16(0xABCD));
  
	let non_secure_characteristic = service
	  .lock()
	  .create_characteristic(BleUuid::Uuid16(0x1234), NimbleProperties::READ);
	non_secure_characteristic
	  .lock()
	  .set_value("non_secure_characteristic".as_bytes());
  
	let secure_characteristic = service.lock().create_characteristic(
	  BleUuid::Uuid16(0x1235),
	  NimbleProperties::READ | NimbleProperties::READ_ENC | NimbleProperties::READ_AUTHEN,
	);
	secure_characteristic
	  .lock()
	  .set_value("secure_characteristic".as_bytes());
  
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

  */