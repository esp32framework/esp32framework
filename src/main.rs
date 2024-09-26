use esp32framework::{ble::BleId, Microcontroller};
use esp32_nimble::*;
use utilities::BleUuid;

fn main(){

    let mut micro = Microcontroller::new();
    let mut client = micro.ble_client();
    client.connect_to_device_with_service(None, &esp32framework::ble::BleId::FromUuid16(0x5678)).unwrap();


    let ble_nimble = &BleUuid::Uuid32(0x12345678);
    println!("Se crea un bleid de nimble con valor {:?}", ble_nimble);
    let ble_nosotros = BleId::from(ble_nimble);
    println!("Se crea un bleid nuestro con valor {:?}", ble_nosotros);
    let traduccion = ble_nosotros.to_uuid();
    println!("Al volver a traducirlo obtenemos {:?}", traduccion);
    
    micro.wait_for_updates(None);
}
