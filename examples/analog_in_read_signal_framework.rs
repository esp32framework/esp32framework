//! Example using pin GPIO5 as analog in to read the intensity of a signal.
//! This value is obtained by reading 'SAMPLING_QUANTITY' times and then 
//! printing its average.

use esp32framework::Microcontroller;

const SAMPLING_QUANTITY: u16 = 10;

fn main(){
    let mut micro = Microcontroller::new().unwrap();
    let mut analog_in = micro.set_pin_as_analog_in_high_atten(5).unwrap();
    
    loop {
        let smooth_read = analog_in.smooth_read(SAMPLING_QUANTITY);
        println!("ADC value with {:?} amount of reads: {:?}",SAMPLING_QUANTITY, smooth_read);
        micro.wait_for_updates(Some(1000));
    }
}
