//! Example using Bluetooth Low energy, a led and the DS3231 sensor.
//! Every 5 seconds, the program takes the temperatur from the ds3231 and advertises
//! it through the BLE beacon. At the same time, a lede on pin 15 blinks twice.

use esp32framework::{
    ble::{
        utils::{ble_standard_uuids::StandarServiceId, Service},
        BleBeacon, BleId,
    },
    gpio::digital::DigitalOut,
    sensors::{DateTime, DS3231},
    Microcontroller,
};

const LED: usize = 15;
const SDA: usize = 5;
const SCL: usize = 6;

fn setup_ds3231<'a>(micro: &mut Microcontroller<'a>) -> DS3231<'a> {
    let i2c = micro.set_pins_for_i2c_master(SDA, SCL).unwrap();
    let mut ds3231 = DS3231::new(i2c);
    let date_time = DateTime {
        second: 5,
        minute: 10,
        hour: 20,
        week_day: 5,
        date: 29,
        month: 8,
        year: 24,
    };

    ds3231.set_time(date_time).unwrap();
    ds3231
}

fn setup_ble_beacon<'a>(micro: &mut Microcontroller<'a>) -> (BleBeacon<'a>, Service) {
    let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
    let mut service = vec![Service::new(&service_id, vec![]).unwrap()];
    let mut beacon = micro
        .ble_beacon("ESP32-beacon".to_string(), &service)
        .unwrap();
    beacon.advertise_service_data(&service_id).unwrap();
    beacon.start().unwrap();
    (beacon, service.pop().unwrap())
}

fn setup_led<'a>(micro: &mut Microcontroller<'a>) -> DigitalOut<'a> {
    let mut led = micro.set_pin_as_digital_out(LED).unwrap();
    led.set_low().unwrap();
    led
}

fn parse_temperature(temp: f32) -> Vec<u8> {
    let int_part = temp as u8;
    let decimal_part = (temp.fract() * 100.0) as u8;
    vec![decimal_part, int_part]
}

fn main() {
    let mut micro = Microcontroller::new();
    let mut ds3231 = setup_ds3231(&mut micro);
    let (mut beacon, mut service) = setup_ble_beacon(&mut micro);
    let mut led = setup_led(&mut micro);

    let mut sent: bool = false;
    loop {
        let date_time = ds3231.get_date_time();
        if date_time.second % 5 == 0 {
            if !sent {
                let temp = ds3231.get_temperature();
                println!("Temperature: {:?}", temp);
                service.data = parse_temperature(temp);
                beacon.set_service(&service).unwrap();
                led.blink(2, 500_000).unwrap();
                sent = true;
            }
        } else {
            sent = false;
        }
        micro.wait_for_updates(Some(300));
    }
}
