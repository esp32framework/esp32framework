use std::time::Duration;

use esp32framework::{
    wifi::http::{Http, HttpHeader},
    Microcontroller,
};

const SSID: &str = "Fibertel WiFi589 2.4GHz";
const PASSWORD: &str = "0041675097";
//const URI: &str = "http://192.168.86.37:3000/api/test";
const URI: &str = "https://httpbin.org/put";

fn main() {
    let mut micro = Microcontroller::take();

    // WIFI connection
    let mut wifi = micro.get_wifi_driver().unwrap();
    println!("connecting");
    micro
        .block_on(wifi.connect(
            SSID,
            Some(PASSWORD.to_string()),
            Some(Duration::from_secs(2)),
        ))
        .unwrap();
    println!("connected");

    // HTTP
    let mut buf: [u8; 1024] = [0; 1024];
    let body: String = String::from("{\"test\": \"mate\", \"body\": \"godo\"}");
    let mut client = wifi.get_https_client().unwrap();
    let header = HttpHeader::new(
        esp32framework::wifi::http::HttpHeaderType::ContentType,
        String::from("application/json"),
    );

    client.put(URI, vec![header], Some(body)).unwrap();

    match client.wait_for_response(&mut buf) {
        Ok(size) => {
            let data = std::str::from_utf8(&buf[0..size]);
            match data {
                Ok(res) => println!("The answer was: {:?}", res),
                Err(_) => println!("Error in parse"),
            };
        }
        Err(e) => println!("Error on read: {:?}", e),
    }

    println!("End of example");
    micro.wait_for_updates(None);
}
