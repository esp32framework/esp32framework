//! Example on how to connect to wifi as a client and then using a HttpClient to perform an
//! HTTP GET request to the website http://ifconfig.net/ and then read the answer that
//! should contain the ip address of the device.
//! Note: Change SSID & PASSWORD values before running the example.

use esp_idf_svc::hal::task::block_on;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, prelude::Peripherals},
    http::{
        client::{Configuration, EspHttpConnection},
        Method,
    },
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{
        AsyncWifi, AuthMethod, ClientConfiguration, Configuration as WifiConfiguration, EspWifi,
    },
};
const SSID: &str = "Iphone 8 Diego New";
const PASSWORD: &str = "diegocivini";
const URI: &str = "https://dog.ceo/api/breeds/image/random";

fn main() {
    esp_idf_svc::sys::link_patches();

    // WIFI connection
    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();
    let timer_service = EspTaskTimerService::new().unwrap();

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
        sys_loop,
        timer_service,
    )
    .unwrap();

    let wifi_configuration: WifiConfiguration = WifiConfiguration::Client(ClientConfiguration {
        ssid: SSID.try_into().unwrap(),
        bssid: None,
        auth_method: AuthMethod::WPAWPA2Personal,
        password: PASSWORD.try_into().unwrap(),
        channel: None,
        ..Default::default()
    });
    wifi.set_configuration(&wifi_configuration).unwrap();

    block_on(async {
        wifi.start().await.unwrap();
        wifi.connect().await.unwrap();
        wifi.wait_netif_up().await.unwrap();
    });

    // HTTP
    let mut buf = [0u8; 1024];
    let config: &Configuration = &Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    };
    let mut client = EspHttpConnection::new(config).unwrap();

    let headers = [("accept", "text/plain")];

    client.initiate_request(Method::Get, URI, &headers).unwrap();

    client.initiate_response().unwrap();
    let bytes_read = client.read(&mut buf).unwrap();

    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(res) => println!("The answer was: {:?}", res),
        Err(_) => println!("Error in parse"),
    };

    println!("End of example");
    FreeRtos::delay_ms(u32::MAX);
}
