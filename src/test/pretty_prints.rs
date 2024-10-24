const BLUE_ANSI: (&str, &str) = ("\x1b[34m", "\x1b[0m");
const RED_ANSI: (&str, &str) = ("\x1b[31m" , "\x1b[0m");
const GREEN_ANSI: (&str, &str) = ("\x1b[32m", "\x1b[0m");
const YELLOW_ANSI: (&str, &str) = ("\x1b[33m", "\x1b[0m");

macro_rules! ansi_format {
    ($ansi:expr, $($arg:tt)*) => {{
        format!("{}{}{}", $ansi.0, format!($($arg)*), $ansi.1)
    }};
}

pub fn print_failing_test(test_number: u8, test_name: &str, reason: &str){
    println!("{}", get_test_id(test_number, test_name) + &ansi_format!(RED_ANSI, "failed, returned: {reason}"))
}

pub fn print_passing_test(test_number: u8, test_name: &str){
    println!("{}", get_test_id(test_number, test_name) + &ansi_format!(GREEN_ANSI, "was successfull"))
}

pub fn print_not_executed_test(test_number: u8, test_name: &str, reason: &str){
    println!("{}", get_test_id(test_number, test_name) + &ansi_format!(YELLOW_ANSI, "not executed due to: {reason}"))
}

fn get_test_id(test_number: u8, test_name: &str)-> String{
    ansi_format!(BLUE_ANSI, "Test: {test_number} {test_name} ")
}