
//! Example of using async wifi.
const SSID: &str = "Iphone 8 Diego New";
const PASSWORD: &str = "diegocivini";
use embedded_svc::http::client::Client;
use core::convert::TryInto;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi,AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use esp_idf_svc::http::client::EspHttpConnection;
// use esp_idf_svc::http::client::Client;

use log::info;


//TODO: micro.get_event_loop()

fn main(){
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
        timer_service,
    ).unwrap();

    //micro.get_wifi_driver()
    
    //micro.connect_to_wifi()

    block_on(connect_wifi(&mut wifi));

    let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();

    info!("Wifi DHCP info: {:?}", ip_info);

    info!("Shutting down in 5s...");

    std::thread::sleep(core::time::Duration::from_secs(5));
}

async fn connect_wifi(wifi: &mut AsyncWifi<EspWifi<'static>>){
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None, // MAC address
        auth_method: AuthMethod::WPA2Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });

    wifi.set_configuration(&wifi_configuration).unwrap();

    wifi.start().await.unwrap();
    info!("Wifi started");

    wifi.connect().await.unwrap();
    info!("Wifi connected");

    wifi.wait_netif_up().await.unwrap();
    info!("Wifi netif up");
}

/*
use core::convert::TryInto;

use embedded_svc::{
    http::{client::Client as HttpClient, Method},
    io::Write,
    utils::io,
    wifi::{AuthMethod, ClientConfiguration, Configuration},
use esp32framework::{ble::{ble_client::BleClient, BleId, Characteristic, Descriptor, Service, StandarCharacteristicId, StandarServiceId}, Microcontroller};
use esp32_nimble::BLEDevice;

fn main(){
	let mut micro = Microcontroller::new();

	// IDs
	let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
	let char_id = BleId::StandarCharacteristic(StandarCharacteristicId::ActivityGoal);
	let desc_id = BleId::FromUuid16(32);

	// Descriptor
	let mut desc = Descriptor::new(desc_id, vec![0x0, 0x1]);
	desc.readeable(true);

	// Characteristic
	let mut characteristic: Characteristic = Characteristic::new(char_id, vec![]);
	characteristic.readeable(true).indicatable(true);
  	characteristic.add_descriptor(desc);

	// Service
	let mut service = Service::new(&service_id, vec![]).unwrap();
	service.add_characteristic(characteristic);
	let services = vec![service];
  
  
  
	let mut server = micro.ble_server("Server".to_string(), &services);
	server.start();
	loop {
		micro.wait_for_updates(Some(1000));
	}
}


/*
use esp32_nimble::{BLEAdvertisementData, BLEDevice, DescriptorProperties, NimbleProperties};
use esp32_nimble::{utilities::BleUuid, uuid128, BLEClient};
use esp_idf_svc::hal::{
  delay::FreeRtos, prelude::Peripherals, task::block_on, timer::{TimerConfig, TimerDriver}
};

use esp_idf_svc::hal::{delay::FreeRtos, peripherals::Peripherals};
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

use log::{error, info};


fn main() {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    // Setup Wifi

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
    ).unwrap();

    connect_wifi(&mut wifi);

    let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default()).unwrap());

    // GET
    get_request(&mut client);

    loop {
      FreeRtos::delay_ms(1000);
    }
}

/// Send an HTTP GET request.
fn get_request(client: &mut HttpClient<EspHttpConnection>) {
    // Prepare headers and URL
    let headers = [("accept", ""), ("Accept-Encoding", "gzip, deflate, br"), ("Connection","keep-alive")];
    let url = "http://google.com";

    // Send request
    //
    // Note: If you don't want to pass in any headers, you can also use `client.get(url, headers)`.
    let request = client.request(Method::Get, url, &headers).unwrap();
    info!("-> GET {}", url);
    let mut response = request.submit().unwrap();

    // Process response
    let status = response.status();
    info!("<- {}", status);
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0).unwrap();
    info!("Read {} bytes", bytes_read);
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => info!(
            "Response body (truncated to {} bytes): {:?}",
            buf.len(),
            body_string
        ),
        Err(e) => error!("Error decoding response body: {}", e),
    };
}

fn connect_wifi(wifi: &mut BlockingWifi<EspWifi<'static>>) {
  let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
      ssid: SSID.try_into().unwrap(),
      bssid: None,
      auth_method: AuthMethod::WPA2Personal,
      password: PASSWORD.try_into().unwrap(),
      channel: None,
      ..Default::default()
  });

  wifi.set_configuration(&wifi_configuration).unwrap();

  wifi.start().unwrap();
  info!("Wifi started");

  wifi.connect().unwrap();
  info!("Wifi connected");

  wifi.wait_netif_up().unwrap();
  info!("Wifi netif up");

}

*/


use esp32framework::{wifi::http::{HttpClient, HttpHeader}, Microcontroller};


fn main(){
  let mut micro = Microcontroller::new();
  let mut wifi = micro.get_wifi_driver();

  let f = wifi.connect(SSID, Some(PASSWORD.to_string()));
  
  micro.block_on(f).unwrap();
  println!("Nos conectamos a WIFI");
  
  let mut client = HttpClient::new().unwrap();
  let header = HttpHeader::new(esp32framework::wifi::http::HttpHeaderType::Custom("accept"), "text/plain");
  let headers = vec![header];
  client.get("http://ifconfig.net/", headers).unwrap();
  println!("Enviamos request");
  
  let mut buf: [u8;1024] = [0;1024];
  client.listen_response().unwrap();
  println!("Ya esperamos la respuesta, vamos a leerla");
  match client.read_response(&mut buf){
    Ok(size) =>  {
      println!("Se tiene {:?} bytes", size);
      let data = std::str::from_utf8(&buf[0..size]);
      match data {
        Ok(res) => println!("La respuesta fue: {:?}", res),
        Err(_) => println!("Error in parse"),
      };
    
    },
    Err(e) => println!("Error on read: {:?}", e),
  }
  
  loop {
    micro.sleep(1000);
  }
}
    esp_idf_svc::log::EspLogger::initialize_default();

    let device = BLEDevice::take();
    let ble_advertising = device.get_advertising();

    let server = device.get_server();
    // server.on_connect(|server, desc| {
    //   ::log::info!("Client connected: {:?}", desc);

    //     if server.connected_count() < (esp_idf_svc::sys::CONFIG_BT_NIMBLE_MAX_CONNECTIONS as ) {
    //         ::log::info!("Multi-connect support: start advertising");
    //         ble_advertising.lock().start().unwrap();
    //     }
    // });
    // server.on_disconnect(|_desc, reason| {
    //   ::log::info!("Client disconnected ({:?})", reason);
    //   if let Err(e) = reason {
    //       println!("El error fue {:?}", e.to_string());
    //   }
    // });

    let service = server.create_service(BleUuid::Uuid16(0xABCD));

    let characteristic = service
      .lock()
      .create_characteristic(BleUuid::Uuid16(0xAAAA), NimbleProperties::READ | NimbleProperties::NOTIFY);
    characteristic
      .lock()
      .set_value("non_secure_characteristic".as_bytes());

    //let desc = characteristic.lock().create_descriptor(BleUuid::Uuid16(0x2900), DescriptorProperties::READ);
    characteristic.lock().create_descriptor(BleUuid::Uuid16(0x2911), DescriptorProperties::READ);
	// desc.lock().set_value(&[0x12;1]);

    // With esp32-c3, advertising stops when a device is bonded.
    // (https://github.com/taks/esp32-nimble/issues/70)
    ble_advertising.lock().set_data(
      BLEAdvertisementData::new()
        .name("ESP32-GATT-Server")
        .add_service_uuid(BleUuid::Uuid16(0xABCD)),
    ).unwrap();
    ble_advertising.lock().start().unwrap();

    ::log::info!("bonded_addresses: {:?}", device.bonded_addresses());

    loop {
      	esp_idf_svc::hal::delay::FreeRtos::delay_ms(1000);
    }
}
	*/
