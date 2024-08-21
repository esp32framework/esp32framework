
use esp32_nimble::{utilities::{mutex::Mutex, BleUuid}, BLEAdvertisementData, BLEAdvertising, BLEDevice};


//EJEMPLO BLE CONECTIONLESS SIN FRAMEWORK
fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let ble_device = BLEDevice::take();
    let ble_advertising = ble_device.get_advertising();
 
    let services1: Vec<u16> = vec![1,2];
    let services2: Vec<u16> = vec![4,8];
    let all_services: Vec<u16> = [services1.clone(), services2.clone()].concat();
    let data1: Vec<[u8; 2]> = vec![[1,1],[2,2]];
    let data2: Vec<[u8; 2]> = vec![[4,4],[8,8]];
    let all_data: Vec<[u8;2]> = [data1.clone(), data2.clone()].concat();
    
    println!("Advertising services 1 and 2");
    let mut advertisement = set_advertisement_services(&ble_advertising,&services1);
    ble_advertising.lock().start().unwrap();
    loop_services(ble_advertising, &mut advertisement, &services1, &data1, 10);
    
    println!("Advertising services 1, 2, 4 and 8");
    let mut advertisement = set_advertisement_services(&ble_advertising,&all_services);
    ble_advertising.lock().start().unwrap();
    loop_services(ble_advertising, &mut advertisement, &all_services, &all_data, 10);
    
    println!("Advertising services 4 and 8");
    let mut advertisement = set_advertisement_services(&ble_advertising,&services2);
    ble_advertising.lock().start().unwrap();
    loop_services(ble_advertising, &mut advertisement, &services2, &data2, 10);

    println!("stop");
    ble_advertising.lock().stop().unwrap();
    loop {
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}

fn set_advertisement_services(ble_advertising: &Mutex<BLEAdvertising>, service_id: &Vec<u16>)-> BLEAdvertisementData{
    let mut advertisement = BLEAdvertisementData::new();
    advertisement.name("My beacon");
 
    for i in service_id{
        let uuid = BleUuid::from_uuid16(*i);
        advertisement.add_service_uuid(uuid);
    }
    ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
        &mut advertisement
    ).unwrap();
    advertisement
}

fn loop_services(ble_advertising: &Mutex<BLEAdvertising>, advertisement :&mut BLEAdvertisementData, services: &Vec<u16>, data: &Vec<[u8;2]>, i: usize){
    let mut services = services.iter().zip(data).cycle();
    for _ in 0..i{
        let (service, data) = services.next().unwrap();
        advertisement.service_data(BleUuid::Uuid16(*service), data);
        ble_advertising.lock().advertisement_type(esp32_nimble::enums::ConnMode::Non).set_data(
            advertisement
        ).unwrap();
        esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}