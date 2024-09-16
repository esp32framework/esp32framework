//! Example on how to connect to wifi as a client and then using a HttpsClient to perform an 
//! HTTPS GET request to the website https://dog.ceo/api/breeds/image/random and then read the answer that 
//! should contain a random link of a dog image.

use esp32framework::{wifi::http::{Http, HttpHeader}, Microcontroller};

const SSID: &str = "WIFI_SSID";
const PASSWORD: &str = "WIFI_PASS";
const URI: &str = "https://dog.ceo/api/breeds/image/random";

fn main(){
    let mut micro = Microcontroller::new();

    // WIFI connection
    let mut wifi = micro.get_wifi_driver();
    micro.block_on(wifi.connect(SSID, Some(PASSWORD.to_string()))).unwrap();

    // HTTP
    let mut buf: [u8;1024] = [0;1024];
    let mut client = wifi.get_https_client().unwrap();
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
 
    loop {
        println!("End of example");
        micro.sleep(1000);
    }
}