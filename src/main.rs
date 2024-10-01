use esp32framework::Microcontroller;

fn main(){

    let mut micro = Microcontroller::new();
    let mut client = micro.ble_client();
    client.connect_to_device_with_service(None, &esp32framework::ble::BleId::FromUuid32(0x12345678)).unwrap();
    client.set_connection_settings(u16::MAX, 100, 10, 1000).unwrap();
    micro.wait_for_updates(None);
}
