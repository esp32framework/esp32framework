//! This example creates a secure ble server. This server has one standar service with three characteristics:
//! - Writable characteristic: Uses an id created from a String.
//!     - Readable descriptor: Uses an standard descriptor uuid (ValidRange) to inform the valid values the user can write.
//! - Readable characteristic: Uses a Standar characteristic uuid (BatteryLevel) to inform the level of the battery.
//! - Notifiable characteristic: Uses an id created from a String to notify an integer value.
//!    Since this server is secure, the clients phone must complete a passkey ('001234') to get access to the information.

use std::sync::Arc;

use esp32_nimble::{
    enums::SecurityIOCap,
    utilities::{mutex::Mutex, BleUuid},
    BLEAdvertisementData, BLECharacteristic, BLEDevice, BLEServer, BLEService, NimbleProperties,
};
use esp_idf_svc::hal::delay::FreeRtos;

const PASSWORD: &str = "001234";

fn add_handlers_to_server(server: &mut BLEServer) {
    server.on_connect(|_server, desc| {
        println!("The client {:?} is connected", desc.address());
    });
    server.on_disconnect(|desc, _reason| {
        println!("The client {:?} is disconnected", desc.address());
    });
}

fn set_up_characteristics(service: Arc<Mutex<BLEService>>) -> Vec<Arc<Mutex<BLECharacteristic>>> {
    // IDs
    let writable_char_id = BleUuid::Uuid128([0x01_u8; 16]);
    let readable_char_id = BleUuid::Uuid128([0x02_u8; 16]);
    let notifiable_char_id = BleUuid::Uuid128([0x03_u8; 16]);

    // Structures
    let writable_characteristic = service
        .lock()
        .create_characteristic(writable_char_id, NimbleProperties::WRITE);
    writable_characteristic.lock().set_value(&[0x00]);

    let readable_characteristic = service
        .lock()
        .create_characteristic(readable_char_id, NimbleProperties::READ);
    readable_characteristic.lock().set_value(&[0x38]);

    let notifiable_characteristic = service.lock().create_characteristic(
        notifiable_char_id,
        NimbleProperties::NOTIFY | NimbleProperties::READ,
    );
    notifiable_characteristic.lock().set_value(&[0x00]);

    vec![
        notifiable_characteristic,
        readable_characteristic,
        writable_characteristic,
    ]
}

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let device = BLEDevice::take();
    let ble_advertising = device.get_advertising();

    device
        .security()
        .set_passkey(PASSWORD.parse::<u32>().unwrap())
        .set_io_cap(SecurityIOCap::KeyboardDisplay);

    let server = device.get_server();

    let service_id = BleUuid::Uuid16(0x180F); // The standard id for Battery service
    let service = server.create_service(service_id);

    let characteristics = set_up_characteristics(service);
    let notifiable_characteristic = characteristics[0].clone();
    add_handlers_to_server(server);

    ble_advertising
        .lock()
        .set_data(
            BLEAdvertisementData::new()
                .name("Example Secure Server")
                .add_service_uuid(BleUuid::Uuid16(0xABCD)),
        )
        .unwrap();
    ble_advertising.lock().start().unwrap();

    let mut counter: u8 = 0;
    loop {
        notifiable_characteristic
            .lock()
            .set_value(&[counter])
            .notify();
        FreeRtos::delay_ms(1000);
        counter += 1;
    }
}
