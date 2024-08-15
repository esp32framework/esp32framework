// use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
// use std::format;

// fn main() {
//   esp_idf_svc::sys::link_patches();
//   esp_idf_svc::log::EspLogger::initialize_default();

//   let ble_device = BLEDevice::take();
//   let ble_advertising = ble_device.get_advertising();

//   let server = ble_device.get_server();
//   server.on_connect(|server, desc| {
//     ::log::info!("Client connected: {:?}", desc);

//     server
//       .update_conn_params(desc.conn_handle(), 24, 48, 0, 60)
//       .unwrap();

//     if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
//       ::log::info!("Multi-connect support: start advertising");
//       ble_advertising.lock().start().unwrap();
//     }
//   });

//   server.on_disconnect(|_desc, reason| {
//     ::log::info!("Client disconnected ({:?})", reason);
//   });

//   let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa"));

//   // A static characteristic.
//   let static_characteristic = service.lock().create_characteristic(
//     uuid128!("d4e0e0d0-1a2b-11e9-ab14-d663bd873d93"),
//     NimbleProperties::READ,
//   );
//   static_characteristic
//     .lock()
//     .set_value("Hello, world!".as_bytes());

//   // A characteristic that notifies every second.
//   let notifying_characteristic = service.lock().create_characteristic(
//     uuid128!("a3c87500-8ed3-4bdf-8a39-a01bebede295"),
//     NimbleProperties::READ | NimbleProperties::NOTIFY,
//   );
//   notifying_characteristic.lock().set_value(b"Initial value.");

//   // A writable characteristic.
//   let writable_characteristic = service.lock().create_characteristic(
//     uuid128!("3c9a3f00-8ed3-4bdf-8a39-a01bebede295"),
//     NimbleProperties::READ | NimbleProperties::WRITE,
//   );
//   writable_characteristic
//     .lock()
//     .on_read(move |_, _| {
//       ::log::info!("Read from writable characteristic.");
//     })
//     .on_write(|args| {
//       ::log::info!(
//         "Wrote to writable characteristic: {:?} -> {:?}",
//         args.current_data(),
//         args.recv_data()
//       );
//     });

//   ble_advertising.lock().set_data(
//     BLEAdvertisementData::new()
//       .name("ESP32-GATT-Server")
//       .add_service_uuid(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafafa")),
//   ).unwrap();
//   ble_advertising.lock().start().unwrap();

//   server.ble_gatts_show_local();

//   let mut counter = 0;
//   loop {
//     esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
//     notifying_characteristic
//       .lock()
//       .set_value(format!("Counter: {counter}").as_bytes())
//       .notify();

//     counter += 1;
//   }
// }





use esp32_nimble::{BLEAdvertisementData, BLEDevice, NimbleProperties, uuid128};
use esp32framework::{ble::*, Microcontroller};
use std::{format, time::Duration};


fn main() {
    let mut micro = Microcontroller::new();
    let ble_device = BLEDevice::take();
    let data: Vec<u8> = vec![0x00, 0x01, 0x04, 0x09];
    let service_id: ServiceId = ServiceId::StandardService(StandarServiceId::Battery);
    let service = Service::new(&service_id, data);
    let services: Vec<Service> = vec![service];
    let mut beacon = BleBeacon::new(ble_device, "MATEO :D".to_string(), services);
    beacon.start_advertisement(&service_id, Duration::from_secs(0)).unwrap();

    loop {
        micro.sleep(1000);
    }
}

// 00000001   04 01 00


// EJEMPLO BLE CONECTIONLESS SIN FRAMEWORK
// fn main() {
//     esp_idf_svc::sys::link_patches();
//     esp_idf_svc::log::EspLogger::initialize_default();

//     let ble_device = BLEDevice::take();
//     let ble_advertising1 = ble_device.get_advertising();
    
//     // Configure el servicio y las características que se publicitarán en la publicidad connectionless
//     let service_uuid1 = esp32_nimble::utilities::BleUuid::from_uuid32(1);
//     let service_uuid2 = esp32_nimble::utilities::BleUuid::from_uuid32(2);
    
//     let mut advertisement1 = BLEAdvertisementData::new();
    
//     advertisement1
//         .name("ESP32-Beacon")
//         .add_service_uuid(service_uuid1)
//         .add_service_uuid(service_uuid2)
//         .service_data(esp32_nimble::utilities::BleUuid::from_uuid16(0), &[0x5;1]);

//     // Configura los datos de publicidad
//     ble_advertising1.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
//         &mut advertisement1
//     ).unwrap();

//     // Empieza la publicidad
//     ble_advertising1.lock().start().unwrap();

//     // Se mantiene el dispositivo publicitando indefinidamente
//     let a = vec![(esp32_nimble::utilities::BleUuid::from_uuid16(2), &[2 as u8;1]),(esp32_nimble::utilities::BleUuid::from_uuid16(1), &[1 as u8;1])];
    
//     for service in a.iter().cycle(){
//         advertisement1.service_data(service.0, service.1);
//         ble_advertising1.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
//             &mut advertisement1
//         ).unwrap();
//         esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
//     }
// }






// use esp_idf_svc::hal::delay::BLOCK;
// use esp_idf_svc::hal::gpio;
// use esp_idf_svc::hal::peripherals::Peripherals;
// use esp_idf_svc::hal::prelude::*;
// use esp_idf_svc::hal::uart::*;
// use esp_idf_svc::hal::delay::FreeRtos;


// fn main(){
//     esp_idf_svc::hal::sys::link_patches();

//     let peripherals = Peripherals::take().unwrap();
//     let tx = peripherals.pins.gpio16;
//     let rx = peripherals.pins.gpio17;

//     println!("Starting UART loopback test");
//     let config = config::Config::new().baudrate(Hertz(115_200));
//     let uart = UartDriver::new(
//         peripherals.uart2,
//         tx,
//         rx,
//         Option::<gpio::Gpio0>::None,
//         Option::<gpio::Gpio1>::None,
//         &config,
//     ).unwrap();

//     loop {
//         uart.write(b"mensaje\n").unwrap();
//         println!("Lo escribi");
//         FreeRtos::delay_ms(1000);
//     }
// }