const BLUE_ANSI: &str = "\x1b[34m";
const RED_ANSI: &str = "\x1b[31m" ;
const GREEN_ANSI: &str = "\x1b[32m";
const YELLOW_ANSI: &str = "\x1b[33m";
const LIGHTBLUE_ANSI: &str = "\x1b[96m";
const RESET_ANSI: &str = "\x1b[0m";

macro_rules! ansi_format {
    ($ansi:expr, $($arg:tt)*) => {{
        format!("{}{}{}", $ansi, format!($($arg)*), RESET_ANSI)
    }};
}

pub fn print_beggin_of_test(){
    println!("{}", &ansi_format!(LIGHTBLUE_ANSI, "======================"))
}

pub fn print_end_of_test(){
    println!("{}", &ansi_format!(LIGHTBLUE_ANSI, "======================"))
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