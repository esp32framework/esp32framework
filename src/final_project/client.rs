use crate::{ble::{utils::{ble_standard_uuids::StandarServiceId, Characteristic, RemoteCharacteristic}, BleClient, BleId}, sensors::DS3231, Microcontroller};

const SERVER_NAME: &str = "Server";

fn initialize_ble_client(micro: &mut Microcontroller) -> BleClient {
    let mut ble = micro.ble_client().unwrap();
    println!("Attempting connection with server: Server" );
    let device = ble.find_device(None, |device| {device.name() == SERVER_NAME} ).unwrap();
    ble.connect_to_device(device).unwrap();
    println!("Connected");
    
    //micro.wait_for_updates(Some(1000)); // TODO

    ble
}

fn get_server_characteristic(client: &mut BleClient) -> RemoteCharacteristic {
    let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
    let mut characteristics = client.get_all_characteristics(&service_id).unwrap();
    let characteristic = characteristics.pop().unwrap(); // TODO: We are testing with only one characteristic. If more are added on the server this has to change
    characteristic
}

fn initialize_sensor<'a>(micro: &mut Microcontroller<'a>) -> DS3231<'a> { // TODO: In the meantime we test with ds32311, then change it to a BME280
    let i2c = micro.set_pins_for_i2c_master(5, 6).unwrap();
    let sensor = DS3231::new(i2c);
    sensor
}


fn main() {
    let mut micro = Microcontroller::take();

    let mut client = initialize_ble_client(&mut micro);
    let mut characteristic = get_server_characteristic(&mut client);
    
    let mut sensor = initialize_sensor(&mut micro);
    loop {
        let temp = sensor.get_temperature().unwrap();

        characteristic.write(&[temp as u8]).unwrap(); // TODO: The f32 should be sent in 2 different bytes. One for the whole nomber part and another one for the decimal part
        

        micro.wait_for_updates(Some(3000));
    }

}