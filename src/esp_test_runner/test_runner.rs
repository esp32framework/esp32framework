use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};
use std::panic;

use super::pretty_prints::*;

const TEST_NAMESPACE: &str = "test_ns";
const CURRENT_TEST_LOCATION: &str = "curr_test";
const SUCCESSFULL_TEST_LOCATION: &str = "failed_test";
const SKIPPED_TEST_LOCATION: &str = "skipped_test";

/// Error types related to test operations.
#[derive(Debug)]
enum TestingErrors {
    FailedToGetNvs,
}

#[derive(Debug)]
/// Reasons why a test may fail
pub enum TestExecutionFailures {
    TestFailed,
    BenchTestNotSupported,
    DynamicTestNotSupported,
}

/// Creates an `EspNvs` driver using the using the default partition and the TEST_NAMESPACE.
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
    EspNvs::new(nvs_default_partition, TEST_NAMESPACE, true)
        .map_err(|_| TestingErrors::FailedToGetNvs)
}

/// Removes entries all test entries from the given NVS instance.
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
/// - `panic!`: If an error occured when using the `EspNvs` driver.
fn reset_testing_env(nvs: &mut EspNvs<NvsDefault>) {
    nvs.remove(CURRENT_TEST_LOCATION).unwrap();
    nvs.remove(SUCCESSFULL_TEST_LOCATION).unwrap();
    nvs.remove(SKIPPED_TEST_LOCATION).unwrap();
}

/// Gets the test statistic from the nvs and prints them
///
/// # Panics
///
/// - `panic!`: If an error occured when using the `EspNvs` driver.
fn get_and_print_test_statistics(nvs: &EspNvs<NvsDefault>, test_quantity: usize) {
    let successfull_tests = nvs.get_u8(SUCCESSFULL_TEST_LOCATION).unwrap().unwrap_or(0);
    let skipped_tests = nvs.get_u8(SKIPPED_TEST_LOCATION).unwrap().unwrap_or(0);
    let failed_tests = test_quantity as u8 - successfull_tests - skipped_tests;
    print_tests_statistics(
        test_quantity as u8,
        failed_tests,
        skipped_tests,
        successfull_tests,
    );
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
fn reset_if_finished(nvs: &mut EspNvs<NvsDefault>, curr_test: u8, test_quantity: usize) -> bool {
    let finished = curr_test as usize >= test_quantity;
    if finished {
        print_end_of_tests();
        get_and_print_test_statistics(nvs, test_quantity);
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
/// - `panic!`: If an error occured when using the `EspNvs` driver.
fn execute_next_test<T: Esp32Test>(nvs: &EspNvs<NvsDefault>, tests: &[T], curr_test: u8) {
    nvs.set_u8(CURRENT_TEST_LOCATION, curr_test + 1).unwrap();

    let t = &tests[curr_test as usize];

    print_executing_test(curr_test, t.name());
    set_testing_panic_hook(curr_test, t.name());

    let res = t.execute();

    reset_panic_hook();

    handle_res(nvs, t.name(), res, curr_test);
}

/// Handles the result of the current test.
///
/// # Parameters
///
/// - `nvs`: An `EspNvs` driver to be able to add to the tests statistics
/// - `test_desc`: A reference to the test.
/// - `res`: The result of the test.
/// - `curr_test`: A mutable reference to the current test counter.
///
/// # Panics
///
/// - `panic!`: If an error occured when using the `EspNvs` driver.
fn handle_res(
    nvs: &EspNvs<NvsDefault>,
    test_name: &str,
    res: Result<(), TestExecutionFailures>,
    curr_test: u8,
) {
    match res {
        Ok(_) => {
            print_passing_test(curr_test, test_name);
            add_to_successfull_counter(nvs);
        }
        Err(err) => match err {
            TestExecutionFailures::TestFailed => {
                print_failing_test(curr_test, test_name, "Incorrect return value")
            }
            TestExecutionFailures::BenchTestNotSupported => {
                print_not_executed_test(curr_test, test_name, &format!("{:?}", err));
                add_to_skipped_counter(nvs);
            }
            TestExecutionFailures::DynamicTestNotSupported => {
                print_not_executed_test(curr_test, test_name, &format!("{:?}", err));
                add_to_skipped_counter(nvs);
            }
        },
    }
}

/// Adds one to the amount of successfull tests
///
/// # Arguments
/// - `nvs`: An `EspNvs` driver to be able to add to the tests statistics
///
/// # Panics
///
/// - `panic!`: If an error occured when using the `EspNvs` driver.
fn add_to_successfull_counter(nvs: &EspNvs<NvsDefault>) {
    let failed_counter = nvs.get_u8(SUCCESSFULL_TEST_LOCATION).unwrap().unwrap_or(0);
    nvs.set_u8(SUCCESSFULL_TEST_LOCATION, failed_counter + 1)
        .unwrap();
}

/// Adds one to the amount of skpped tests
///
/// # Arguments
/// - `nvs`: An `EspNvs` driver to be able to add to the tests statistics
///
/// # Panics
///
/// - `panic!`: If an error occured when using the `EspNvs` driver.
fn add_to_skipped_counter(nvs: &EspNvs<NvsDefault>) {
    let failed_counter = nvs.get_u8(SKIPPED_TEST_LOCATION).unwrap().unwrap_or(0);
    nvs.set_u8(SKIPPED_TEST_LOCATION, failed_counter + 1)
        .unwrap();
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
/// - `panic!`: If an error occured when using the `EspNvs` driver.
pub fn esp32_test_runner<T: Esp32Test>(tests: &[T]) {
    print_test_separator();
    let mut nvs = get_nvs().unwrap();

    let curr_test = nvs.get_u8(CURRENT_TEST_LOCATION).unwrap().unwrap_or(0);

    if !reset_if_finished(&mut nvs, curr_test, tests.len()) {
        execute_next_test(&nvs, tests, curr_test);
        print_test_separator();
        unsafe { esp_idf_svc::sys::esp_restart() };
    }
    print_test_separator();
}

/// Extends a test struct with an `execute` method.
pub trait Esp32Test {
    /// Execute the test function.
    ///
    /// # Returns
    ///
    /// A `Result` containing `()` if the execution succeeds, or a `TestExecutionFailure`
    /// if it fails.
    fn execute(&self) -> Result<(), TestExecutionFailures>;

    /// Returns an `&str` that identifies the tets
    fn name(&self) -> &str;
}
