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
use esp_idf_svc::{hal::delay::FreeRtos, nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault}};

const INVALID_PIN_NUM: usize = 100;

//////// MICROCONTROLLER ////////

/*
/// Try to create an analog in pwm with an invalid pin number.
/// Should panic.
fn invalid_pin_num_analog_in_pwm() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_analog_in_pwm(INVALID_PIN_NUM, 10000); // TODO: What freq should we use?
}

/// Try to create an analog in pwm with an invalid pin number using the default creation.
/// Should panic.
fn invalid_pin_num_default_analog_in_pwm() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_default_analog_in_pwm(INVALID_PIN_NUM);
}
 */
/// Try to create an analog in with an invalid pin number using high attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_high_atten() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_analog_in_high_atten(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using medium attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_medium_atten() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_analog_in_medium_atten(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using low attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_low_atten() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_analog_in_low_atten(INVALID_PIN_NUM);
}

/// Try to create an analog in with an invalid pin number using none attenuation.
/// Should panic.
fn invalid_pin_num_analog_in_none_atten() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_analog_in_no_atten(INVALID_PIN_NUM);
}

/// Try to create an analog out with an invalid pin number.
/// Should panic.
fn invalid_pin_num_analog_out() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_analog_out(INVALID_PIN_NUM, 10000, 13); // TODO: What freq or resolution should we use?
}

/// Try to create an analog out with an invalid pin number with default settings.
/// Should panic.
fn invalid_pin_num_analog_out_default() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_default_analog_out(INVALID_PIN_NUM);
}

/// Try to create an digital out with an invalid pin number.
/// Should panic.
fn invalid_pin_num_digital_out() {
    let mut micro = Microcontroller::take();
    micro.set_pin_as_digital_out(INVALID_PIN_NUM);
}

/// Try to create an digital in with an invalid pin number.
/// Should panic.
fn invalid_pin_num_digital_in() {
    panic!("hola");
    let mut micro = Microcontroller::take();
    micro.set_pin_as_digital_in(INVALID_PIN_NUM).unwrap(); //todo ver este error
}

fn passing_test() {
    let mut a = 0;
    a += 1;
}

use std::{env::args, process::{ExitCode, Termination}, sync::{Arc, Mutex}};
use std::panic;
const TEST_QUANTITY: usize = 2;
const TESTS: [fn()->(); TEST_QUANTITY] = [invalid_pin_num_digital_in, passing_test];
const TEST_NAMESPACE: &str = "test_ns";
const CURRENT_TEST_LOCATION: &str = "curr_test";
const LAST_TEST_LOCATION: &str = "last_test";

type SharableNvs = Arc<Mutex<EspNvs<NvsDefault>>>;

#[derive(Debug)]
enum TestingErrors{
    FailedToGetNvs,
    DataFailure
}

fn get_nvs() -> Result<SharableNvs, TestingErrors> {
    let nvs_default_partition: EspNvsPartition<NvsDefault> = EspDefaultNvsPartition::take().map_err(|_| TestingErrors::FailedToGetNvs)?;
    Ok(Arc::new(Mutex::from(EspNvs::new(nvs_default_partition, TEST_NAMESPACE, true).map_err(|_| TestingErrors::FailedToGetNvs)?)))
}

fn reset_testing_env(nvs: &SharableNvs){
    let mut nvs = nvs.lock().unwrap();
    nvs.remove(CURRENT_TEST_LOCATION).map_err(|_| TestingErrors::DataFailure).unwrap();
    nvs.remove(LAST_TEST_LOCATION).map_err(|_| TestingErrors::DataFailure).unwrap();
}

fn reset_in_case_of_error(nvs: &SharableNvs, curr_test: &mut u8){
    let last_test = nvs.lock().unwrap().get_i16(LAST_TEST_LOCATION).unwrap().unwrap_or(-1);
    if last_test + 1 != *curr_test as i16 {
        reset_testing_env(nvs);
        *curr_test = 0;
    }
}

fn block_if_finished(nvs: &SharableNvs, curr_test: u8) {
    if curr_test as usize >= TEST_QUANTITY{
        println!("End of tests");
        reset_testing_env(nvs);
        FreeRtos::delay_ms(u32::MAX);
    }
}

fn set_testing_panic_hook(nvs: &SharableNvs, curr_test: u8 ) {
    let hook = panic::take_hook();
    
    let nvs = nvs.clone();
    panic::set_hook(Box::new(move |panic_info| {
        println!("\x1b[31m Test {curr_test} failed \x1b[0m");
        hook(panic_info);
        nvs.lock().unwrap().set_i16(LAST_TEST_LOCATION, curr_test as i16).map_err(|_| TestingErrors::DataFailure).unwrap();
        unsafe { esp_idf_svc::sys::esp_restart() };
    }));
}

fn reset_panic_hook(){
    _ = panic::take_hook();
}

fn execute_next_test(nvs: &SharableNvs, curr_test: u8) {
    println!("Executing test {curr_test}");
    nvs.lock().unwrap().set_u8(CURRENT_TEST_LOCATION, curr_test + 1).map_err(|_| TestingErrors::DataFailure).unwrap();
    
    set_testing_panic_hook(nvs, curr_test);

    TESTS[curr_test as usize]();

    reset_panic_hook();

    nvs.lock().unwrap().set_i16(LAST_TEST_LOCATION, curr_test as i16).map_err(|_| TestingErrors::DataFailure).unwrap();

    println!("\x1b[32m Test {curr_test} was successfull \x1b[0m");
}

fn main() {
    let nvs = get_nvs().unwrap();
    
    let mut curr_test = nvs.lock().unwrap().get_u8(CURRENT_TEST_LOCATION).unwrap().unwrap_or(0);

    reset_in_case_of_error(&nvs, &mut curr_test);
    
    block_if_finished(&nvs, curr_test);

    if curr_test == 0 {
        FreeRtos::delay_ms(5);
    }

    execute_next_test(&nvs, curr_test);

    unsafe { esp_idf_svc::sys::esp_restart() };
}

