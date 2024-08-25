use esp32_nimble::{uuid128, BLEAdvertisementData, BLEDevice, NimbleProperties};
use esp32framework::{
    ble::{BleId, BleServer, Characteristic, Service}, InterruptDriver, Microcontroller
    
};

fn main(){
    let mut micro = Microcontroller::new();

    let mut i = 0;
    let mut service_1 = Service::new(&BleId::FromUuid32(21), vec![0x65, 0x45]).unwrap();
    let service_2 = Service::new(&BleId::FromUuid32(32), vec![]).unwrap();
    let mut server = micro.ble_server("DIEGO".to_string(), vec![service_1, service_2]);
    // server.set_service(&service_1.clone()).unwrap();
    // server.set_service(&service_2.clone()).unwrap();
    
    server.connection_handler(move |server, info| {
        println!("Se conecto {:?} !!!!!!!!", info.address);
        // if i == 0{
        //     server.set_characteristic(BleId::FromUuid32(21), &Characteristic::new(BleId::ByName("caract".to_string()),vec![0x0;5])).unwrap();
            
        // }else if i == 1{
        //     server.set_characteristic(BleId::FromUuid32(32), &Characteristic::new(BleId::ByName("caract_2".to_string()),vec![0x1;5])).unwrap();
        // }

        // i += 1;
    });

    // TESTING!
    server.disconnect_handler(move |_, info| {
        println!("Se desconecto {:?}!!!!!!!!", info.address);
    });


    server.start().unwrap();
    server.set_characteristic(BleId::FromUuid32(21), &Characteristic::new(BleId::ByName("caract_2".to_string()),vec![0x1;5])).unwrap();

    loop{
        server.update_interrupt().unwrap();
        micro.sleep(100);
    }
} 


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

//     // if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as _) {
//     //   ::log::info!("Multi-connect support: start advertising");
//     //   ble_advertising.lock().start().unwrap();
//     // }
//   });

// //   server.on_disconnect(|_desc, reason| {
// //     ::log::info!("Client disconnected ({:?})", reason);
// //   });

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

//   ble_advertising.lock().start();
//   ble_advertising.lock().stop();
//   let service = server.create_service(uuid128!("fafafafa-fafa-fafa-fafa-fafafafafaff"));
//   ble_advertising.lock().start();

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