// BLE MAIN CON FRAMEWORK 
/*
use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp32framework::{ble::*, Microcontroller};
use std::{format, time::Duration};


fn main() {
  let mut micro = Microcontroller::new();
  let ble_device = BLEDevice::take();
  // let ble_advertising = ble_device.get_advertising();
  let data: Vec<u8> = vec![];
  let service_id: BleId = BleId::ByName("Diego service".to_string());
  let mut service = Service::new(&service_id, data.clone()).unwrap();
  let characteristic_id: BleId = BleId::ByName("Diego char".to_string());
  let mut my_characteristic: Characteristic = Characteristic::new(characteristic_id, vec![]);
  my_characteristic.writable(true).readeable(true);
  let mut char2 = Characteristic::new(BleId::ByName("2nd service".to_string()), vec![0x01, 0x02]);
  char2.notifiable(true).readeable(true);

  service.add_characteristic(my_characteristic);
  service.add_characteristic(char2);

  let mut ble_server = BleServer::new(String::from("Diego"), ble_device, vec![service]);
  let handler = |info: &ConnectionInformation| { println!("Se conecto el usuario: {:?}", info.address())};
  ble_server.connection_handler(move |arg0: &ConnectionInformation| handler(arg0));
  ble_server.start().unwrap();

  micro.wait_for_updates(None);
}

// BLE MAIN
*/

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



//SINGLE SERVICE EXAMPLE
/*
use esp32framework::{ble::*, Microcontroller};
use std::{format, time::Duration};


// fn main() {
//     let mut micro = Microcontroller::new();
//     let ble_device = BLEDevice::take();
//     let data: Vec<u8> = vec![];
//     let service_id: ServiceId = ServiceId::ByName("Diego".to_string());
//     // let service_id2: ServiceId = ServiceId::ByName("Mateo".to_string());
//     // let service_id3: ServiceId = ServiceId::ByName("Diego".to_string());
//     // let service_id4: ServiceId = ServiceId::ByName("Diego".to_string());
//     println!("El service id Diego es: {:?}", service_id.to_uuid());
//     // println!("El service id Diego es: {:?}", service_id3.to_uuid());
//     // println!("El service id Diego es: {:?}", service_id4.to_uuid());
//     // println!("El service id Mateo es: {:?}", service_id2.to_uuid());

//     let service = Service::new(&service_id, data).unwrap();
//     let services: Vec<Service> = vec![service];
//     let mut beacon = BleBeacon::new(ble_device, "MATEO".to_string(), services).unwrap();
//     beacon.start().unwrap();

//     loop {
//         micro.sleep(1000);
//     }
// }

*/
// 00000001   04 01 00




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

// BLE_BEACON FRAMEWORK
/*
use esp32framework::{ ble::{Service, BleId}, Microcontroller};

fn main(){
    let mut micro = Microcontroller::new();
    let mut services1 = vec![];
    let mut services2 = vec![];
    for i in 1..3{
        services1.push(Service::new(&BleId::FromUuid16(i as u16), vec![i;2]).unwrap());
        services2.push(Service::new(&BleId::FromUuid16((i*4) as u16), vec![i*4;2]).unwrap());
    }
    let mut beacon = micro.ble_beacon("Mateo lindo".to_string(), &services1);
    
    beacon.set_services(&services1).unwrap();
    beacon.advertise_all_service_data().unwrap();
    beacon.start().unwrap();
    
    println!("Advertising services 1 and 2");
    micro.wait_for_updates(Some(10000));
    
    println!("Advertising services 1, 2, 4 and 8");
    beacon.set_services(&services2).unwrap();
    micro.wait_for_updates(Some(10000));
    
    println!("Advertising services 4 and 8");
    beacon.remove_services(&vec![BleId::FromUuid16(1), BleId::FromUuid16(2)]).unwrap();
    micro.wait_for_updates(Some(10000));
    
    println!("stop");
    beacon.stop().unwrap();
    micro.wait_for_updates(None);
    }
*/

use esp32_nimble::{utilities::{mutex::Mutex, BleUuid}, BLEAdvertisementData, BLEAdvertising, BLEDevice};


//EJEMPLO BLE CONECTIONLESS SIN FRAMEWORK
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();
 
    let services1: Vec<u16> = vec![1,2];
    let services2: Vec<u16> = vec![4,8];
    let all_services: Vec<u16> = [services1.clone(), services2.clone()].concat();
    let data1: Vec<[u8; 2]> = vec![[1,1],[2,2]];
    let data2: Vec<[u8; 2]> = vec![[4,4],[8,8]];
    let all_data: Vec<[u8;2]> = [data1.clone(), data2.clone()].concat();
    
    println!("Advertising services 1 and 2");
    let mut advertisement = set_advertisement_services(&ble_advertising,&services1);
    ble_advertising.lock().start().unwrap();
    loop_services(ble_advertising, &mut advertisement, &services1, &data1, 10);
    
    println!("Advertising services 1, 2, 4 and 8");
    let mut advertisement = set_advertisement_services(&ble_advertising,&all_services);
    ble_advertising.lock().start().unwrap();
    loop_services(ble_advertising, &mut advertisement, &all_services, &all_data, 10);
    
    println!("Advertising services 4 and 8");
    let mut advertisement = set_advertisement_services(&ble_advertising,&services2);
    ble_advertising.lock().start().unwrap();
    loop_services(ble_advertising, &mut advertisement, &services2, &data2, 10);

    println!("stop");
    ble_advertising.lock().stop().unwrap();
    loop {
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}

fn set_advertisement_services(ble_advertising: &Mutex<BLEAdvertising>, service_id: &Vec<u16>)-> BLEAdvertisementData{
    let mut advertisement = BLEAdvertisementData::new();
    advertisement.name("My beacon");
 
    for i in service_id{
        let uuid = BleUuid::from_uuid16(*i);
        advertisement.add_service_uuid(uuid);
    }
    ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
        &mut advertisement
    ).unwrap();
    advertisement
}

fn loop_services(ble_advertising: &Mutex<BLEAdvertising>, advertisement :&mut BLEAdvertisementData, services: &Vec<u16>, data: &Vec<[u8;2]>, i: usize){
    let mut services = services.iter().zip(data).cycle();
    for _ in 0..i{
        let (service, data) = services.next().unwrap();
        advertisement.service_data(BleUuid::Uuid16(*service), data);
        ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
            advertisement
        ).unwrap();
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}