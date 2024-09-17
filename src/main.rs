use esp32framework::{ble::BleId, Microcontroller};

fn main(){
  let mut micro = Microcontroller::new();
  let mut ble_client = micro.ble_client();
  micro.ble_beacon("hola".to_string(), &vec![]);
  let service_id = BleId::FromUuid32(0x12345678);
  ble_client.connect_to_device_with_service(None, &service_id).unwrap();
  let mut c = ble_client.get_characteristic(&service_id, &BleId::FromUuid16(0x0101)).unwrap();
  
  loop{
    let mut d = c.get_descriptor(&BleId::FromUuid16(0x0404)).unwrap();
    //println!("Read: {:?}", d.read().unwrap());
    d.write(&[0,0,0]).unwrap();
    println!("write");
    micro.wait_for_updates(None);
    let a = c.read().unwrap();
  }
}