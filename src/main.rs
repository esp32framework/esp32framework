/*
The '-' means is already done
The '+' means is not done

# Microcontroller
- No se puede crear un analog in pwm pin con un numero invalido
- No se puede crear un analog in pwm default pin con un numero invalido
- No se puede crear un analog in low atten pin con un numero invalido
- No se puede crear un analog in medium atten pin con un numero invalido
- No se puede crear un analog in high atten pin con un numero invalido
- No se puede crear un analog in no atten pin con un numero invalido
- No se puede crear un analog out pin con un numero invalido
- No se puede crear un analog out default pin con un numero invalido
- No se puede crear un digital out pin con un numero invalido
- No se puede crear un digital in pin con un numero invalido
+ No se puede crear un i2c master con un numero invalido de pin sds o scl
+ No se puede crear un i2c slave con un numero invalido de pin sds o scl
+ No se puede crear un uart con un numero invalido de pin tx o rx, o un uart num invalido
+ No se puede crear un uart default con un numero invalido de pin tx o rx, o un uart num invalido


*/

use esp32framework::{wifi::http::{Http, HttpHeader}, Microcontroller};
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};

const INVALID_PIN_NUM: usize = 100;

//////// MICROCONTROLLER ////////

/// Try to create an analog in pwm with an invalid pin number.
/// Should panic.
fn invalid_pin_num_analog_in_pwm(micro: &mut Microcontroller) {
    micro.set_pin_as_analog_in_pwm(INVALID_PIN_NUM, 10000); // TODO: What freq should we use?
}

/// Try to create an analog in pwm with an invalid pin number using the default creation.
/// Should panic.
fn invalid_pin_num_default_analog_in_pwm(micro: &mut Microcontroller) {
    micro.set_pin_as_default_analog_in_pwm(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using high attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_high_atten(micro: &mut Microcontroller) {
    micro.set_pin_as_analog_in_high_atten(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using medium attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_medium_atten(micro: &mut Microcontroller) {
    micro.set_pin_as_analog_in_medium_atten(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using low attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_low_atten(micro: &mut Microcontroller) {
    micro.set_pin_as_analog_in_low_atten(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using none attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_none_atten(micro: &mut Microcontroller) {
    micro.set_pin_as_analog_in_no_atten(INVALID_PIN_NUM);
}

/// Try to create an analog out with an invalid pin number.
/// Should panic.
fn invalid_pin_num_analog_out(micro: &mut Microcontroller) {
    micro.set_pin_as_analog_out(INVALID_PIN_NUM, 10000, 13); // TODO: What freq or resolution should we use?
}

/// Try to create an analog out with an invalid pin number with default settings.
/// Should panic.
fn invalid_pin_num_analog_out_default(micro: &mut Microcontroller) {
    micro.set_pin_as_default_analog_out(INVALID_PIN_NUM);
}

/// Try to create an digital out with an invalid pin number.
/// Should panic.
fn invalid_pin_num_digital_out(micro: &mut Microcontroller) {
    micro.set_pin_as_digital_out(INVALID_PIN_NUM);
}

/// Try to create an digital in with an invalid pin number.
/// Should panic.
fn invalid_pin_num_digital_in(micro: &mut Microcontroller) {
    micro.set_pin_as_digital_in(INVALID_PIN_NUM);
}



fn main(){
    let mut micro = Microcontroller::new();
    
    let nvs_default_partition: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take().unwrap();
    let test_namespace = "test_ns";
    let nvs = match EspNvs::new(nvs_default_partition, test_namespace, true) {
        Ok(nvs) => nvs,
        Err(e) => panic!("Could't get namespace {:?}", e),
    };

    let counter_name = "counter";

    let counter_new_value: u8 = match nvs.get_u8(counter_name).unwrap() {
        Some(v) => {
            println!("{:?} = {:?}", counter_name, v);
            v + 1
        }
        None => 0,
    };

    match nvs.set_u8(counter_name, counter_new_value) {
        Ok(_) => println!("Counter updated"),
        Err(e) => println!("Counter not updated {:?}", e),
    };

    loop {

        if counter_new_value == 0 {
            invalid_pin_num_analog_in_pwm(&mut micro);
        } else if counter_new_value == 1 {
            invalid_pin_num_default_analog_in_pwm(&mut micro);
        } else if counter_new_value == 2 {
            invalid_pin_num_analog_in_high_atten(&mut micro);
        }

        println!("End of example");
        
        esp_idf_svc::hal::reset::restart();
    }
}

