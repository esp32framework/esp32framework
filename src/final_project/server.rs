use crate::{ble::{utils::{ble_standard_uuids::{StandarCharacteristicId, StandarServiceId}, Characteristic, Service}, BleId}, Microcontroller};

const ADVERTISING_NAME: &str = "Server";
const SSID: &str = "WIFI_SSID";
const PASSWORD: &str = "WIFI_PASS";

fn initialize_ble_server(micro: &mut Microcontroller) {
    let characteristic_id = BleId::StandarCharacteristic(StandarCharacteristicId::TemperatureMeasurement);
    let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
    
    let characteristic = Characteristic::new(characteristic_id, vec![]);
    let mut service = Service::new(&service_id, vec![0xAB]).unwrap(); // fix initial data
    service.add_characteristic(characteristic);
    let services = vec![service];
    
    let mut server = micro.ble_server(ADVERTISING_NAME.to_string(), &services).unwrap(); // TODO: Check if we want a secure server
    
    server.start().unwrap();
}


fn main() {
    let mut micro = Microcontroller::take();
    initialize_ble_server(&mut micro);
    
    
    micro.wait_for_updates(None);
}