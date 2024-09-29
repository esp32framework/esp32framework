//! Example on how to connect to wifi as a client and then using a HttpClient to perform an 
//! HTTP GET request to the website http://ifconfig.net/ and then read the answer that 
//! should contain the ip address of the device.
//! Note: Change SSID & PASSWORD values before running the example.

use esp32framework::{wifi::http::{Http, HttpHeader}, Microcontroller};

const SSID: &str = "WIFI_SSID";
const PASSWORD: &str = "WIFI_PASS";
const URI: &str = "http://ifconfig.net/";

fn main(){
    let mut micro = Microcontroller::new().unwrap();

    // WIFI connection
    let mut wifi = micro.get_wifi_driver();
    micro.block_on(wifi.connect(SSID, Some(PASSWORD.to_string()))).unwrap();

    // HTTP
    let mut buf: [u8;1024] = [0;1024];
    let mut client = wifi.get_http_client().unwrap();
    let header = HttpHeader::new(esp32framework::wifi::http::HttpHeaderType::Accept, "text/plain");
    
    client.get(URI, vec![header]).unwrap();

    match client.wait_for_response(&mut buf) {
        Ok(size) =>  {
        let data = std::str::from_utf8(&buf[0..size]);
        match data {
            Ok(res) => println!("The answer was: {:?}", res),
            Err(_) => println!("Error in parse"),
        };
        },
        Err(e) => println!("Error on read: {:?}", e),
    }
 
    println!("End of example");
    micro.wait_for_updates(None);
}
