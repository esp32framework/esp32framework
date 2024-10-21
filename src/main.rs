// use esp32framework::Microcontroller;

// fn main() {
//     let mut micro = Microcontroller::take();

//     // WIFI scan
//     let mut wifi = micro.get_wifi_driver().unwrap();

//     let resutls = micro
//         .block_on(wifi.scan())
//         .unwrap();

//     for acces_point in resutls.iter(){
//         println!("SSID: {:?}", acces_point.ssid);
//         println!("Auth: {:?}", acces_point.authentication_method);
//         println!("Signal: {:?}", acces_point.signal_strength);
//         println!("-----------------------------------------");
//     }

//     println!("End of example");
//     micro.wait_for_updates(None);
// }
