use esp32framework::sensors::HCSR04;
use esp32framework::Microcontroller;

fn main(){

    let mut micro = Microcontroller::new();
    let echo = micro.set_pin_as_digital_in(6);
    let trig = micro.set_pin_as_digital_out(5);
    let mut sensor = HCSR04::new(trig, echo);
    

    // let delay = Delay::new_default();
    
    loop {
        let distance = sensor.get_distance();
        println!("{:?} cm", distance);
        micro.sleep(1000);
    }
}
