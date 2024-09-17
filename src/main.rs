use esp32framework::{ble::{BleId, Service}, Microcontroller};

fn main(){
  let mut micro = Microcontroller::new();
  let mut beacon = micro.ble_beacon("hola".to_string(), &vec![]);
  for i in 0..100{
    beacon.set_service(&Service::new(&BleId::FromUuid16(i), vec![]).unwrap()).unwrap();
  }
}