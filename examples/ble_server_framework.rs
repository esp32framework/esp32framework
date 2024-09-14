//! This example creates a secure ble server. This server has one standar service with three characteristics:
//! - Writable characteristic: Uses an id created from a String.
//! - Readable characteristic: Uses a Standar characteristic uuid (BatteryLevel) to inform the level of the battery.
//! - Notifiable characteristic: Uses an id created from a String to notify an integer value.
//! Since this server is secure, the clients phone must complete a passkey ('001234') to get access to the information.

use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp32framework::{ble::{BleId, BleServer, Characteristic, IOCapabilities, Security, Service, StandarCharacteristicId, StandarServiceId}, InterruptDriver, Microcontroller};
 


// fn main() {
// 	esp_idf_svc::sys::link_patches();
// 	esp_idf_svc::log::EspLogger::initialize_default();
  
// 	let ble_device = BLEDevice::take();
// 	let ble_advertising = ble_device.get_advertising();
  
// 	let server = ble_device.get_server();
  
// 	let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));
  
//     // A characteristic that notifies every second.
//     let notifying_characteristic = service.lock().create_characteristic(
//       uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
//       NimbleProperties::READ | NimbleProperties::NOTIFY,
//     );
//     notifying_characteristic.lock().set_value(b"Initial value.");
  
  
//   ble_advertising.lock().set_data(
// 	BLEAdvertisementData::new()
// 	  .name("ESP32-GATT-Server")
// 	  .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa")),
//   ).unwrap();
  
// 	// let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafaff"));
// 	ble_advertising.lock().start();
  
// 	server.ble_gatts_show_local();
  
// 	let mut counter = 0;
// 	loop {
// 	  esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
// 	  notifying_characteristic
// 	    .lock()
// 	    .set_value(format!("Counter: {counter}").as_bytes())
// 	    .notify();
  
// 	  counter += 1;
// 	}
  
//   }



fn set_up_characteristics() -> Vec<Characteristic> {
	// IDs
    let writable_char_id = BleId::FromUuid16(32);
	let readable_char_id = BleId::StandarCharacteristic(StandarCharacteristicId::BatteryLevel);
	//let notifiable_char_id = BleId::ByName("NotifiableCharacteristic".to_string());
	let notifiable_char_id = BleId::FromUuid16(32);

	// Structures
    let mut writable_characteristic = Characteristic::new(writable_char_id, vec![0x00]);
    writable_characteristic.writable(true);

	let mut readable_characteristic = Characteristic::new(readable_char_id, vec![0x38]);
	readable_characteristic.readeable(true);

    let mut notifiable_characteristic = Characteristic::new(notifiable_char_id, vec![0x10]);
	notifiable_characteristic.readeable(true).notifiable(true);

	vec![/*readable_characteristic, writable_characteristic,*/ notifiable_characteristic]
}

fn add_handlers_to_server(server: &mut BleServer){
	server.connection_handler(| _server, connection_info| { 
        println!("The client {:?} is connected", connection_info.address)
    });
	server.disconnect_handler(| _server, connection_info| { 
        println!("The client {:?} is disconnected", connection_info.address)
    });
}

fn main(){
	let mut micro = Microcontroller::new();
	
	// Security configuration
	// let phone_capabilities = IOCapabilities::KeyboardDisplay;
	// let security = Security::new(001234,phone_capabilities);
    
	let mut characteristics: Vec<Characteristic> = set_up_characteristics();
    let service_id = BleId::StandardService(StandarServiceId::Battery);
    let mut service = Service::new(&service_id, vec![0xAB]).unwrap();
	service.add_characteristics(&characteristics);
    
	//let mut server = micro.ble_secure_server("Example Secure Server".to_string(), &vec![service], security);
    let mut server = micro.ble_server("Example Secure Server".to_string(), &vec![service]);

	add_handlers_to_server(&mut server);

	server.start().unwrap();
	
    let mut counter: u8 = 1;
    loop {
        characteristics[0].update_data(vec![counter; 2]);
        server.notify_value(service_id.clone(), &characteristics[0]).unwrap();
		micro.wait_for_updates(Some(10000));
        counter += 1;
	}
}