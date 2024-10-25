use esp_idf_svc::{
    hal::delay::FreeRtos,
    nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault},
};
use std::panic;
use std::{
    process::{ExitCode, Termination},
    sync::{Arc, Mutex},
};
use super::pretty_prints::*;

const TEST_NAMESPACE: &str = "test_ns";
const CURRENT_TEST_LOCATION: &str = "curr_test";

extern crate test;

/// Error types related to test operations.
#[derive(Debug)]
enum TestingErrors {
    FailedToGetNvs,
    DataFailure,
}

#[derive(Debug)]
enum TestExecutionErrors{
    TestFailed(String),
    BenchTestNotSupported,
    DynamicTestNotSupported,
}

/// Creates a shared, mutable reference to the NVS partition using default 
/// settings and the TEST_NAMESPACE.
///
/// # Returns
///
/// A `Result` containing an `Arc<Mutex<EspNvs>>` representing the initialized NVS 
/// or an `TestingErrors` if it fails.
///
/// # Errors
///
/// - `TestingErrors::FailedToGetNvs`: If there's an issue retrieving or initializing the NVS.
fn get_nvs() -> Result<EspNvs<NvsDefault>, TestingErrors> {
    let nvs_default_partition: EspNvsPartition<NvsDefault> =
        EspDefaultNvsPartition::take().map_err(|_| TestingErrors::FailedToGetNvs)?;
    Ok(EspNvs::new(nvs_default_partition, TEST_NAMESPACE, true).map_err(|_| TestingErrors::FailedToGetNvs)?)
}

/// Removes entries for CURRENT_TEST_LOCATION and LAST_TEST_LOCATION
/// from the given NVS instance.
///
/// # Parameters
///
/// - `nvs`: A reference to the NVS testing instance.
///
/// # Errors
///
/// - `TestingErrors::DataFailure`: If there's an issue removing data from the NVS.
///
/// # Panics
///
/// - `panic!`: If the lock on the NVS cannot be acquired.
fn reset_testing_env(nvs: &mut EspNvs<NvsDefault>) {
    nvs.remove(CURRENT_TEST_LOCATION).unwrap();
}

/// Checks if the current test is the last and resets the testing 
/// environment if so.
///
/// # Parameters
///
/// - `nvs`: A reference to the NVS testing instance.
/// - `curr_test`: A mutable reference to the current test counter.
/// - `test_quantity`: The total number of tests.
///
/// # Returns
///
/// `true` if the tests have ended,`false` otherwise.
fn reset_if_finished(nvs: &mut EspNvs<NvsDefault>, curr_test: u8, test_quantity: usize)->bool{
    let finished = curr_test as usize >= test_quantity;
    if  finished{
        print_end_of_tests();
        reset_testing_env(nvs);
    }
    finished
}

/// Sets the panic hook in ordet to do custom printing of testing information 
/// and to set the information in the NVS.
///
/// # Parameters
///
/// - `nvs`: A reference to the NVS testing instance.
/// - `curr_test`: A mutable reference to the current test counter.
/// - `test_name`: The name of the current test.
///
/// # Panics
///
/// - `panic!`: If the lock on the NVS cannot be acquired.
fn set_testing_panic_hook(curr_test: u8, test_name: &str) {
    let hook = panic::take_hook();
    let test_name = String::from(test_name);
    panic::set_hook(Box::new(move |panic_info| {
        print_failing_test(curr_test, &test_name, "pannicked");
        hook(panic_info);
        print_test_separator();
        unsafe { esp_idf_svc::sys::esp_restart() };
    }));
}

/// Restores the original panic hook taken at the beginning of the testing session.
fn reset_panic_hook() {
    _ = panic::take_hook();
}

/// Executes the next test, updating the NVS information and seting the current 
/// panic hook.
///
/// # Parameters
///
/// - `nvs`: A reference to the NVS testing instance.
/// - `tests`: A slice of tests.
/// - `curr_test`: A mutable reference to the current test counter.
///
/// # Panics
///
/// - `panic!`: If the lock on the NVS cannot be acquired.
fn execute_next_test(nvs: &EspNvs<NvsDefault>   , tests: &[&test::TestDescAndFn], curr_test: u8) {
    nvs.set_u8(CURRENT_TEST_LOCATION, curr_test + 1).unwrap();

    let t = tests[curr_test as usize];

    print_executing_test(curr_test, t.desc.name.as_slice());
    set_testing_panic_hook(curr_test, t.desc.name.as_slice());

    let res = t.execute();

    reset_panic_hook();

    handle_res(&t.desc, res, curr_test);
}

/// Handles the result of the current test.
///
/// # Parameters
///
/// - `test_desc`: A reference to the test.
/// - `res`: The result of the test.
/// - `curr_test`: A mutable reference to the current test counter.
fn handle_res(test_desc: &test::TestDesc, res: Result<(), TestExecutionErrors>, curr_test: u8){
    match res{
        Ok(_) => print_passing_test(curr_test, test_desc.name.as_slice()),
        Err(err) => match err{
            TestExecutionErrors::TestFailed(reason) => print_failing_test(curr_test, test_desc.name.as_slice(), &reason),
            TestExecutionErrors::BenchTestNotSupported => print_not_executed_test(curr_test, test_desc.name.as_slice(), &format!("{:?}", err)),
            TestExecutionErrors::DynamicTestNotSupported => print_not_executed_test(curr_test, test_desc.name.as_slice(), &format!("{:?}", err)),
        },
    }
}

/// Custom Test Runner for ESP32 Tests. This runner restarts the ESP before executing 
/// each test to ensure a clean test environment. To archive this goal, it 
/// uses the NVS (Non-Volatile Storage), so user tests cannot access this resource
/// 
/// NOTE: Testing FLAGS are not supported.
///
/// # Parameters
///
/// - `tests`: A slice of tests to be run.
///
/// # Panics
///
/// - `panic!`: If the lock on the NVS cannot be acquired.
pub fn esp32_test_runner(tests: &[&test::TestDescAndFn]) {
    print_test_separator();
    let mut nvs = get_nvs().unwrap();

    let curr_test = nvs.get_u8(CURRENT_TEST_LOCATION).unwrap().unwrap_or(0);

    if !reset_if_finished(&mut nvs, curr_test, tests.len()){
        execute_next_test(&nvs, tests, curr_test);
        print_test_separator();
        unsafe { esp_idf_svc::sys::esp_restart() };
    }
    print_test_separator();
}

/// Extends a test struct with an `execute` method.
trait TestExecutionExtention{
    /// Execute the test function.
    /// 
    /// # Returns
    ///
    /// A `Result` containing `()` if the execution succeeds, or a `TestingErrors`
    /// if it fails.
    fn execute(&self)-> Result<(), TestExecutionErrors>;
}

impl TestExecutionExtention for test::TestDescAndFn{
    fn execute(&self)-> Result<(), TestExecutionErrors> {
        self.testfn.execute()
    }
}

impl TestExecutionExtention for test::TestFn{
    fn execute(&self)-> Result<(), TestExecutionErrors>{
        match self{
            test::TestFn::StaticTestFn(func) => func().map_err(TestExecutionErrors::TestFailed),
            test::TestFn::DynTestFn(fn_once) => Err(TestExecutionErrors::DynamicTestNotSupported),
            _ => Err(TestExecutionErrors::BenchTestNotSupported)
        }
    }
}