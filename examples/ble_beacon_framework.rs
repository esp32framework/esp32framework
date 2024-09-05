

use esp32framework::{ ble::{Service, BleId}, Microcontroller};

fn main(){
    let mut micro = Microcontroller::new();
    let mut services1 = vec![];
    let mut services2 = vec![];
    for i in 1..3{
        services1.push(Service::new(&BleId::FromUuid16(i as u16), vec![i;2]).unwrap());
        services2.push(Service::new(&BleId::FromUuid16((i*4) as u16), vec![i*4;2]).unwrap());
    }
    let mut beacon = micro.ble_beacon("My Beacon".to_string(), &services1);
    
    beacon.advertise_all_service_data().unwrap();
    beacon.start().unwrap();
    
    println!("Advertising services 1 and 2");
    micro.wait_for_updates(Some(10000));
    
    println!("Advertising services 1, 2, 4 and 8");
    beacon.set_services(&services2).unwrap();
    micro.wait_for_updates(Some(10000));
    
    println!("Advertising services 4 and 8");
    beacon.remove_services(&vec![BleId::FromUuid16(1), BleId::FromUuid16(2)]).unwrap();
    micro.wait_for_updates(Some(10000));
    
    println!("stop");
    beacon.stop().unwrap();
    micro.wait_for_updates(None);
}