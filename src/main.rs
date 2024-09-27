use esp32framework::Microcontroller;

fn main(){

    let mut micro = Microcontroller::new();
    let mut client = micro.ble_client();
    client.connect_to_device_with_service(None, &esp32framework::ble::BleId::FromUuid16(0x5678)).unwrap();
   
    micro.wait_for_updates(None);
}
