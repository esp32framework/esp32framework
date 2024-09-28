
use esp32framework::Microcontroller;

const SERVICE_UUID: u16 = 0x0101;

fn main(){
    let mut micro = Microcontroller::new();
    let mut client = micro.ble_client();
    let device = client.find_device_with_service(None, &esp32framework::ble::BleId::FromUuid16(SERVICE_UUID)).unwrap();

    client.connect_to_device(device).unwrap();
    micro.wait_for_updates(None);
}
