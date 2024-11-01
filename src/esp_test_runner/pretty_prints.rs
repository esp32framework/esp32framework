const BLUE_ANSI: &str = "\x1b[34m";
const RED_ANSI: &str = "\x1b[31m";
const GREEN_ANSI: &str = "\x1b[32m";
const YELLOW_ANSI: &str = "\x1b[33m";
const LIGHTBLUE_ANSI: &str = "\x1b[96m";
const RESET_ANSI: &str = "\x1b[0m";
const BROWN_ANSI: &str = "\x1b[38;5;94m";
const TEST_SEPARATOR: &str = "======================";

/// Wrapper of format macro to color the output string.0m
///
/// # Returns
/// A `String` colored acording to the received ansi code
macro_rules! ansi_format {
    ($ansi:expr, $($arg:tt)*) => {{
        format!("{}{}{}", $ansi, format!($($arg)*), RESET_ANSI)
    }};
}

/// Print the executing test identification
pub fn print_executing_test(test_number: u8, test_name: &str) {
    println!(
        "{} ...",
        &(ansi_format!(BLUE_ANSI, "Executing ") + &get_test_id(BLUE_ANSI, test_number, test_name))
    )
}

/// Prints the end of tests
pub fn print_end_of_tests() {
    println!("{}", &ansi_format!(BROWN_ANSI, "Finished Executing tests"))
}

/// Prints the `TEST_SEPARATOR`
pub fn print_test_separator() {
    println!("{}", &ansi_format!(LIGHTBLUE_ANSI, "{}", TEST_SEPARATOR))
}

/// Prints the received test identification, that it failed upon execution and the reason why it failed
pub fn print_failing_test(test_number: u8, test_name: &str, reason: &str) {
    println!(
        "{}",
        get_test_id(BLUE_ANSI, test_number, test_name)
            + &ansi_format!(RED_ANSI, "failed, {reason}")
    )
}

/// Prints the received test identification, that it succeded upon execution
pub fn print_passing_test(test_number: u8, test_name: &str) {
    println!(
        "{}",
        get_test_id(BLUE_ANSI, test_number, test_name)
            + &ansi_format!(GREEN_ANSI, "was successfull")
    )
}

/// Prints the received test identification, that it was not executed, and the reason it was not executed
pub fn print_not_executed_test(test_number: u8, test_name: &str, reason: &str) {
    println!(
        "{}",
        get_test_id(BLUE_ANSI, test_number, test_name)
            + &ansi_format!(YELLOW_ANSI, "not executed due to: {reason}")
    )
}

/// Prints the rest statistics
pub fn print_tests_statistics(
    test_quantity: u8,
    failed_tests: u8,
    skipped_tests: u8,
    successfull_tests: u8,
) {
    println!(
        "{}: {} | {} | {}",
        ansi_format!(BROWN_ANSI, "Test quantity {test_quantity}"),
        ansi_format!(GREEN_ANSI, "Successfull tests: {successfull_tests}"),
        ansi_format!(RED_ANSI, "Failed tests: {failed_tests}"),
        ansi_format!(YELLOW_ANSI, "Skipped tests: {skipped_tests}"),
    )
}

/// Returns the string for the test identification in the given ansi color
fn get_test_id(color: &str, test_number: u8, test_name: &str) -> String {
    ansi_format!(color, "Test: {test_number} ") + test_name + " "
}
