//! This example creates a secure ble server. This server has one standar service with three characteristics:
//! - Writable characteristic: Uses an id created from a String.
//! - Readable characteristic: Uses a Standar characteristic uuid (BatteryLevel) to inform the level of the battery.
//! - Notifiable characteristic: Uses an id created from a String to notify an integer value.
//!    Since this server is secure, the clients phone must complete a passkey ('001234') to get access to the information.

use esp32framework::{
    ble::{
        utils::{
            ble_standard_uuids::{StandarCharacteristicId, StandarServiceId},
            Characteristic, IOCapabilities, Security, Service,
        },
        BleId, BleServer,
    },
    Microcontroller,
};

const PASSWORD: &str = "001234";

fn set_up_characteristics() -> Vec<Characteristic> {
    // IDs
    let writable_char_id = BleId::FromUuid128([0x01; 16]);
    let readable_char_id = BleId::StandarCharacteristic(StandarCharacteristicId::BatteryLevel);
    let notifiable_char_id = BleId::FromUuid128([0x02; 16]);

    // Structures
    let mut writable_characteristic = Characteristic::new(writable_char_id, vec![0x00]);
    writable_characteristic.writable(true);

    let mut readable_characteristic = Characteristic::new(readable_char_id, vec![0x38]);
    readable_characteristic.readeable(true);

    let mut notifiable_characteristic = Characteristic::new(notifiable_char_id, vec![0x10]);
    notifiable_characteristic.readeable(true).notifiable(true);

    vec![
        notifiable_characteristic,
        readable_characteristic,
        writable_characteristic,
    ]
}

fn add_handlers_to_server(server: &mut BleServer) {
    server.connection_handler(|_server, connection_info| {
        println!("The client {:?} is connected", connection_info.address)
    });
    server.disconnect_handler(|_server, connection_info| {
        println!("The client {:?} is disconnected", connection_info.address)
    });
}

fn main() {
    let mut micro = Microcontroller::new();

    // Security configuration
    let phone_capabilities = IOCapabilities::DisplayOnly;
    let security = Security::new(PASSWORD.parse::<u32>().unwrap(), phone_capabilities).unwrap();

    let characteristics: Vec<Characteristic> = set_up_characteristics();
    let mut notifiable_characteristic = characteristics[0].clone();
    let service_id = BleId::StandardService(StandarServiceId::Battery);
    let mut service = Service::new(&service_id, vec![0xAB]).unwrap();
    service.add_characteristics(&characteristics);

    let mut server = micro
        .ble_secure_server(
            "Example Secure Server".to_string(),
            &vec![service],
            security,
        )
        .unwrap();

    add_handlers_to_server(&mut server);

    server.start().unwrap();

    let mut counter: u8 = 1;
    loop {
        notifiable_characteristic.update_data(vec![counter]);
        server
            .notify_value(service_id.clone(), &notifiable_characteristic)
            .unwrap();
        micro.wait_for_updates(Some(1000));
        counter += 1;
    }
}
