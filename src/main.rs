use esp32framework::{sensors::DS3231, serial::READER, Microcontroller};
fn main() {
    let mut micro = Microcontroller::new();
    let i2c = micro.set_pins_for_i2c_master(5, 6).unwrap();
    let mut ds3231: DS3231<'_> = DS3231::new(i2c);

    loop{
        ds3231.show_data("secs".to_string()).unwrap();
        // let data = ds3231.read_n_times_and_sum("secs".to_string(), 4, 1000).unwrap();
        // let data = ds3231.read_n_times_and_avg("secs".to_string(), 4, 1000).unwrap();

        let data = ds3231.read_n_times_and_aggregate("secs".to_string(), 4, 1000, |data| {
            println!("{:?}", data)
        }).unwrap();
        
        // ds3231.show_data("secs".to_string()).unwrap();
        // println!("data: {}", data);

        micro.sleep(500);
    }
}
