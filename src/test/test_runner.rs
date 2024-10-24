use esp_idf_svc::{
    hal::delay::FreeRtos,
    nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault},
};
use std::panic;
use std::{
    process::{ExitCode, Termination},
    sync::{Arc, Mutex},
};

use super::pretty_pints::{print_failing_test, print_not_executed_test, print_passing_test};
const TEST_NAMESPACE: &str = "test_ns";
const CURRENT_TEST_LOCATION: &str = "curr_test";
const LAST_TEST_LOCATION: &str = "last_test";

extern crate test;

type SharableNvs = Arc<Mutex<EspNvs<NvsDefault>>>;

#[derive(Debug)]
enum TestingErrors {
    TestFailed(String),
    BenchTestNotSupported,
    DynamicTestNotSupported,
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

fn reset_if_finished(nvs: &SharableNvs, curr_test: u8, test_quantity: usize)->bool{
    let finished = curr_test as usize >= test_quantity;
    if  finished{
        println!("End of tests");
        reset_testing_env(nvs);
    }
    finished
}

fn set_testing_panic_hook(nvs: &SharableNvs, curr_test: u8, test_name: String) {
    let hook = panic::take_hook();

    let nvs = nvs.clone();
    panic::set_hook(Box::new(move |panic_info| {
        print_failing_test(curr_test, &test_name, "panniced");
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

fn execute_next_test(nvs: &SharableNvs, tests: &[&test::TestDescAndFn], curr_test: u8) {
    nvs.lock()
        .unwrap()
        .set_u8(CURRENT_TEST_LOCATION, curr_test + 1)
        .map_err(|_| TestingErrors::DataFailure)
        .unwrap();

    let t = tests[curr_test as usize];

    println!("Executing test{curr_test}: {}", t.desc.name);
    set_testing_panic_hook(nvs, curr_test, String::from(t.desc.name.as_slice()));

    let res = t.execute();

    reset_panic_hook();

    nvs.lock()
        .unwrap()
        .set_i16(LAST_TEST_LOCATION, curr_test as i16)
        .map_err(|_| TestingErrors::DataFailure)
        .unwrap();

    handle_res(&t.desc, res, curr_test);
}

fn handle_res(test_desc: &test::TestDesc, res: Result<(), TestingErrors>, curr_test: u8){
    match res{
        Ok(_) => print_passing_test(curr_test, test_desc.name.as_slice()),
        Err(err) => match err{
            TestingErrors::TestFailed(reason) => print_failing_test(curr_test, test_desc.name.as_slice(), &reason),
            TestingErrors::BenchTestNotSupported => print_not_executed_test(curr_test, test_desc.name.as_slice(), &format!("{:?}", err)),
            TestingErrors::DynamicTestNotSupported => print_not_executed_test(curr_test, test_desc.name.as_slice(), &format!("{:?}", err)),
            _ => Err(err).unwrap(),
        },
    }
}

pub fn esp32_test_runner(tests: &[&test::TestDescAndFn]) {
    let nvs = get_nvs().unwrap();

    let mut curr_test = nvs
        .lock()
        .unwrap()
        .get_u8(CURRENT_TEST_LOCATION)
        .unwrap()
        .unwrap_or(0);

    reset_in_case_of_error(&nvs, &mut curr_test);

    if !reset_if_finished(&nvs, curr_test, tests.len()){
        execute_next_test(&nvs, tests, curr_test);

        unsafe { esp_idf_svc::sys::esp_restart() };
    }
}

trait TestExecutionExtention{
    fn execute(&self)-> Result<(), TestingErrors>;
}

impl TestExecutionExtention for test::TestDescAndFn{
    fn execute(&self)-> Result<(), TestingErrors> {
        self.testfn.execute()
    }
}

impl TestExecutionExtention for test::TestFn{
    fn execute(&self)-> Result<(), TestingErrors>{
        match self{
            test::TestFn::StaticTestFn(func) => func().map_err(TestingErrors::TestFailed),
            test::TestFn::DynTestFn(fn_once) => Err(TestingErrors::DynamicTestNotSupported),
            _ => Err(TestingErrors::BenchTestNotSupported)
        }
    }
}