//! This example creates a secure ble server. This server has one standar service with three characteristics:
//! - Writable characteristic: Uses an id created from a String.
//! - Readable characteristic: Uses a Standar characteristic uuid (BatteryLevel) to inform the level of the battery.
//! - Notifiable characteristic: Uses an id created from a String to notify an integer value.
//!    Since this server is secure, the clients phone must complete a passkey ('001234') to get access to the information.

use esp32framework::{
    ble::{
        utils::{
            ble_standard_uuids::{StandarCharacteristicId, StandarDescriptorId, StandarServiceId}, Characteristic, Descriptor, Service
        },
        BleId, BleServer,
    },
    Microcontroller,
};


fn main() {
    let mut micro = Microcontroller::take();
    
    let characteristic_id = BleId::StandarCharacteristic(StandarCharacteristicId::Temperature);
    let mut characteristic = Characteristic::new(&characteristic_id, vec![0x00]);
    characteristic.readable(true).writable(true).indicatable(true);

    let descriptor_id = BleId::StandarDescriptor(StandarDescriptorId::EnvironmentalSensingMeasurement);
    let descriptor = Descriptor::new(descriptor_id, vec![0x01]);

    characteristic.add_descriptor(descriptor);
                
    
    let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
    let mut service = Service::new(&service_id, vec![0xAB]).unwrap();
    
    service.add_characteristic(&characteristic);

    let mut server = micro
        .ble_server(
            "Server".to_string(),
            &vec![service],
        )
        .unwrap();

    server.start().unwrap();

    loop {
        
        micro.wait_for_updates(Some(2000));

        let read_val = server.get_characteristic_data(&service_id, &characteristic_id).unwrap();

        println!("El server leyo: {:?}", read_val);
    }
}
