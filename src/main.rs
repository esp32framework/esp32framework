use esp32framework::{
    esp32_framework_error::Esp32FrameworkError,
    wifi::http::{Http, HttpHeader},
    Microcontroller,
};
use esp_idf_svc::hal::gpio::Pull;

const SSID: &str = "Fibertel WiFi588 2.4GHz";
const PASSWORD: &str = "0041675097";
//const URI: &str = "http://192.168.86.37:3000/api/test";
const URI: &str = "https://httpbin.org/put";

fn main() -> Result<(), Esp32FrameworkError> {
    let mut micro = Microcontroller::take();

    let mut dgin = micro.set_pin_as_digital_in(2)?;
    dgin.set_pull(Pull::Down)?;

    loop {
        println!("Is high {}", dgin.is_high());
        micro.sleep(1000);
    }
}
