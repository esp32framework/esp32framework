use esp_idf_svc::{
    hal::delay::FreeRtos,
    nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault},
};
use std::panic;
use std::{
    process::{ExitCode, Termination},
    sync::{Arc, Mutex},
};
const TEST_NAMESPACE: &str = "test_ns";
const CURRENT_TEST_LOCATION: &str = "curr_test";
const LAST_TEST_LOCATION: &str = "last_test";

type SharableNvs = Arc<Mutex<EspNvs<NvsDefault>>>;

#[derive(Debug)]
enum TestingErrors {
    FailedToGetNvs,
    DataFailure,
}

fn get_nvs() -> Result<SharableNvs, TestingErrors> {
    let nvs_default_partition: EspNvsPartition<NvsDefault> =
        EspDefaultNvsPartition::take().map_err(|_| TestingErrors::FailedToGetNvs)?;
    Ok(Arc::new(Mutex::from(
        EspNvs::new(nvs_default_partition, TEST_NAMESPACE, true)
            .map_err(|_| TestingErrors::FailedToGetNvs)?,
    )))
}

fn reset_testing_env(nvs: &SharableNvs) {
    let mut nvs = nvs.lock().unwrap();
    nvs.remove(CURRENT_TEST_LOCATION)
        .map_err(|_| TestingErrors::DataFailure)
        .unwrap();
    nvs.remove(LAST_TEST_LOCATION)
        .map_err(|_| TestingErrors::DataFailure)
        .unwrap();
}

fn reset_in_case_of_error(nvs: &SharableNvs, curr_test: &mut u8) {
    let last_test = nvs
        .lock()
        .unwrap()
        .get_i16(LAST_TEST_LOCATION)
        .unwrap()
        .unwrap_or(-1);
    if last_test + 1 != *curr_test as i16 {
        reset_testing_env(nvs);
        *curr_test = 0;
    }
}

fn block_if_finished(nvs: &SharableNvs, curr_test: u8, test_quantity: usize) {
    if curr_test as usize >= test_quantity {
        println!("End of tests");
        reset_testing_env(nvs);
        FreeRtos::delay_ms(u32::MAX);
    }
}

fn set_testing_panic_hook(nvs: &SharableNvs, curr_test: u8) {
    let hook = panic::take_hook();

    let nvs = nvs.clone();
    panic::set_hook(Box::new(move |panic_info| {
        println!("\x1b[31m Test {curr_test} failed \x1b[0m");
        hook(panic_info);
        nvs.lock()
            .unwrap()
            .set_i16(LAST_TEST_LOCATION, curr_test as i16)
            .map_err(|_| TestingErrors::DataFailure)
            .unwrap();
        unsafe { esp_idf_svc::sys::esp_restart() };
    }));
}

fn reset_panic_hook() {
    _ = panic::take_hook();
}

fn execute_next_test(nvs: &SharableNvs, tests: &[&(&'static str, fn())], curr_test: u8) {
    nvs.lock()
        .unwrap()
        .set_u8(CURRENT_TEST_LOCATION, curr_test + 1)
        .map_err(|_| TestingErrors::DataFailure)
        .unwrap();
    let (name, func) = tests[curr_test as usize];

    println!("Executing test{curr_test}: {name}");
    set_testing_panic_hook(nvs, curr_test);

    func();

    reset_panic_hook();

    nvs.lock()
        .unwrap()
        .set_i16(LAST_TEST_LOCATION, curr_test as i16)
        .map_err(|_| TestingErrors::DataFailure)
        .unwrap();

    println!("\x1b[32m Test {curr_test}: {name} was successfull \x1b[0m");
}

pub fn esp32_test_runner(tests: &[&(&'static str, fn())]) {
    let nvs = get_nvs().unwrap();

    let mut curr_test = nvs
        .lock()
        .unwrap()
        .get_u8(CURRENT_TEST_LOCATION)
        .unwrap()
        .unwrap_or(0);

    reset_in_case_of_error(&nvs, &mut curr_test);

    block_if_finished(&nvs, curr_test, tests.len());

    execute_next_test(&nvs, tests, curr_test);

    unsafe { esp_idf_svc::sys::esp_restart() };
}
