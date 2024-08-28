// Leer del reloj temp y tiempo, cada x tiempo mandar la temp
// Mandar info por ble
// prender led cuando se manda

use esp32framework::{ble::{BleBeacon, BleId, Service, StandarCharacteristicId, StandarServiceId}, gpio::DigitalOut, sensors::{DateTime, DS3231}, Microcontroller};

const SDA: usize = 5;
const SCL: usize = 6;
const LED: usize = 15;

fn setup_ds3231<'a>(micro: &mut Microcontroller<'a>)-> DS3231<'a>{
  let i2c = micro.set_pins_for_i2c_master(SDA,SCL);
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

fn setup_ble_beacon<'a>(micro: & mut Microcontroller<'a>)-> (BleBeacon<'a>, Service) {
  let service_id = BleId::StandardService(StandarServiceId::EnvironmentalSensing);
  let mut service = vec![Service::new(&service_id, vec![]).unwrap()];
  let mut beacon = micro.ble_beacon("ESP32-beacon".to_string(), &service);
  beacon.advertise_service_data(&service_id).unwrap();
  beacon.start().unwrap();
  (beacon, service.pop().unwrap())
}

fn setup_led<'a>(micro: & mut Microcontroller<'a>)-> DigitalOut<'a>{
  let mut led = micro.set_pin_as_digital_out(LED);
  led.set_low().unwrap();
  led
}

fn main(){
  let mut micro = Microcontroller::new();
  let mut ds3231 = setup_ds3231(&mut micro);
  let (mut beacon, mut service) = setup_ble_beacon(&mut micro);
  let mut led = setup_led(&mut micro);

  let mut sent: bool = false;
  loop {
    let date_time = ds3231.get_date_time();
    if date_time.second % 10 == 0 {
      if !sent{
        let temp = ds3231.get_temperature();
		println!("Temperature: {:?}", temp);

        service.data = parse_temp(temp); // parsear la temp
        beacon.set_service(&service).unwrap();
        led.blink(2, 500_000).unwrap();
        sent = true;
      }
    }else{
        sent = false;
    }
    micro.wait_for_updates(Some(300));
  }
}

fn parse_temp(temp: f32) -> Vec<u8> {
  let int_part = temp as u8;
  let decimal_part = ((temp - int_part as f32) * 100 as f32) as u8;
  vec![decimal_part, int_part]

}

/*
fn parse_temp(temp: f32) -> Vec<u8> {
  let bits = temp.to_bits();
  let bytes = bits.to_le_bytes();
  bytes.to_vec()
}

fn main() {
    let temp = 32.123456;
    let parsed = parse_temp(temp);
    println!("{:?}", parsed);

    let a = f32::from_le_bytes(parsed.try_into().unwrap());
    println!("{:?}", a);
}

 */
