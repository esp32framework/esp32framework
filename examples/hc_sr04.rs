//! Example to use the HC-SR04 without framework. Every second it gets the centimeters of the
//! object that is in front.

use esp_idf_svc::{
    hal::{delay::Delay, gpio::PinDriver, prelude::Peripherals},
    sys::esp_timer_get_time,
};

const SOUND_SPEED_M_S: f64 = 340.0;
const SOUND_SPEED_CM_US: f64 = SOUND_SPEED_M_S * 100.0 / 1_000_000.0;

fn main() {
    let peripherals = Peripherals::take().unwrap();
    let echo = PinDriver::input(peripherals.pins.gpio6).unwrap();
    let mut trig = PinDriver::output(peripherals.pins.gpio5).unwrap();

    let delay = Delay::new_default();

    loop {
        // Activate the trigger so the echo is sent
        trig.set_low().unwrap();
        delay.delay_us(4);
        trig.set_high().unwrap();
        delay.delay_us(10);
        trig.set_low().unwrap();

        while echo.is_low() {}

        // Get the moment when the echo was sent
        let send_echo_time = unsafe { esp_timer_get_time() };

        // Wait for the echo to bounce back
        while echo.is_high() {}

        // Get the moment when the echo arrived
        let rec_echo_time = unsafe { esp_timer_get_time() };

        let travel_time = rec_echo_time - send_echo_time;
        let cm: f64 = SOUND_SPEED_CM_US * travel_time as f64;
        let distance = cm / 2.0; // We divide by 2 because if not we get the distance of the roundtrip

        println!("{:?} cm", distance);

        delay.delay_ms(1000);
    }
}
