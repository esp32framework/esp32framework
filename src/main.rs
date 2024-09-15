
//! Example of using async wifi.
const SSID: &str = "Iphone 8 Diego New";
const PASSWORD: &str = "diegocivini";
use embedded_svc::http::client::Client;
use esp32framework::wifi::http::HttpHeader;
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
/*
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

// ASYNC WIFI EXAMPLE WITH FRAMEWORK
use esp32framework::Microcontroller;

fn main(){
  let mut micro = Microcontroller::new();
  let mut wifi = micro.get_wifi_driver();

  let f = wifi.connect(SSID, Some(PASSWORD.to_string()));
  
  micro.block_on(f).unwrap();
  println!("Nos conectamos a WIFI");
  
  let mut client = wifi.get_http_client().unwrap();
  let header = HttpHeader::new(esp32framework::wifi::http::HttpHeaderType::Accept, "text/plain");
  let headers = vec![header];
  client.get("http://ifconfig.net/", headers).unwrap();
  println!("Enviamos request");
  
  println!("Mi ip es {:?}",wifi.get_address_info());
  println!("Mi dns es {:?}",wifi.get_dns_info());

  let mut buf: [u8;1024] = [0;1024];
  println!("Ya esperamos la respuesta, vamos a leerla");
  match client.wait_for_response(&mut buf) {
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

