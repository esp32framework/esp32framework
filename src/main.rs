use esp32framework::{ble::{utils::{ble_standard_uuids::{StandarCharacteristicId, StandarServiceId}, Characteristic, Service}, BleId, BleServer}, wifi::http::HttpClient, Microcontroller};

const ADVERTISING_NAME: &str = "Server";
const SSID: &str = "WIFI_SSID";
const PASSWORD: &str = "WIFI_PASS";
const URI: &str = "web application uri";

fn initialize_ble_server<'a>(micro: &mut Microcontroller<'a>) -> BleServer<'a> {
    let characteristic_id = BleId::StandarCharacteristic(StandarCharacteristicId::Temperature);
    let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
    
    let characteristic = Characteristic::new(&characteristic_id, vec![]);
    let mut service = Service::new(&service_id, vec![0xAB]).unwrap(); // fix initial data
    service.add_characteristic(&characteristic);
    let services = vec![service];
    
    let server = micro.ble_server(ADVERTISING_NAME.to_string(), &services).unwrap();
    server
}

fn initialize_wifi_connection(micro: &mut Microcontroller) -> HttpClient {
    let mut wifi = micro.get_wifi_driver().unwrap();
    micro
        .block_on(wifi.connect(SSID, Some(PASSWORD.to_string())))
        .unwrap();
    
    wifi.get_http_client().unwrap()
}

/// Gathers data from the connected devices
fn gather_data(server: &BleServer) -> Vec<String> {
    vec![]
}

/// Sends the collected data of the devices to the web application
/// so they can be shown to the users
fn send_data(client: &HttpClient, data: Vec<String>) {

}

fn main() {
    let mut micro = Microcontroller::take();

    let server = initialize_ble_server(&mut micro);
    let client = initialize_wifi_connection(&mut micro);
    
    
    loop {
        micro.wait_for_updates(Some(5000));

        let data = gather_data(&server);
        send_data(&client, data);
    }

}
