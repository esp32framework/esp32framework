
//! Example of using async wifi.
const SSID: &str = "Iphone 8 Diego New";
const PASSWORD: &str = "diegocivini";
/*
use core::convert::TryInto;
use esp_idf_svc::hal::prelude::Peripherals;
use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::timer::EspTaskTimerService;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi,AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};

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
 */

use esp32framework::{wifi::wifi::WifiDriver, Microcontroller};


fn main(){
  let mut micro = Microcontroller::new();
  let mut wifi = micro.get_wifi_driver();

  let f = wifi.connect(SSID, Some(PASSWORD.to_string()));

  micro.block_on(f);
  println!("Pasamos");

  loop {
    micro.sleep(1000);
  }
}