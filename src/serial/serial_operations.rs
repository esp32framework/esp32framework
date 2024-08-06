use std::{collections::HashMap, thread::panicking};

use esp_idf_svc::{hal::delay::FreeRtos, sys::configTICK_RATE_HZ};

pub enum SerialError {
    ErrorInReadValue,
}


pub trait READER {
    fn read_and_parse(& mut self) -> HashMap<String, String>;
}

pub trait WRITER { 
    fn parse_and_write(&mut self, addr: u8, bytes_to_write: &[u8]) -> Result<(), SerialError>;
}

pub fn show_data(data_reader: &mut impl READER, operation_key: String) -> Result<(), SerialError> {
    let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
    match parsed_data.get(&operation_key) {
        Some(data) => println!("The content is: {:?}", data),
        None => {return Err(SerialError::ErrorInReadValue);}
    }
    Ok(())
}

// TODO: Document that we work with floats (values with "," will explode)
pub fn read_n_times_and_sum(data_reader: &mut impl READER, operation_key: String, times: usize, ms_between_reads: u32) -> Result<f32, SerialError> {
    let mut total = 0.0;
    for _ in 0..times {
        let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
        match parsed_data.get(&operation_key) {
            Some(data) =>  total += data.parse::<f32>().map_err(|_| SerialError::ErrorInReadValue)?,
            None => {return Err(SerialError::ErrorInReadValue)}
        }
        FreeRtos::delay_ms(ms_between_reads);
    }
    Ok(total)
}

// TODO: Document that we work with floats (values with "," will explode)
pub fn read_n_times_and_avg(data_reader: &mut impl READER, operation_key: String, times: usize, ms_between_reads: u32) -> Result<f32, SerialError> {
    let mut total = 0.0;
    for _ in 0..times {
        let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
        match parsed_data.get(&operation_key) {
            Some(data) =>  total += data.parse::<f32>().map_err(|_| SerialError::ErrorInReadValue)?,
            None => {return Err(SerialError::ErrorInReadValue)}
        }
        FreeRtos::delay_ms(ms_between_reads);
    }
    Ok(total / (times as f32))
}

pub fn read_n_times_and_aggregate<C, T>(data_reader: &mut impl READER, operation_key: String, times: usize, ms_between_reads: u32, execute_closure: C) -> Result<T, SerialError>
where
C: Fn(Vec<String>) -> T
{
    let mut read_values: Vec<String> = vec![];
    for _ in 0..times {
        let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
        match parsed_data.get(&operation_key) {
            Some(data) =>  {
                println!("{:?}", data);
                read_values.push(data.clone());
            },
            None => {return Err(SerialError::ErrorInReadValue)}
        }
        FreeRtos::delay_ms(ms_between_reads);
    }
    Ok(execute_closure(read_values))
}

pub fn execute_when_true<C1, C2>(data_reader: &mut impl READER, operation_key: String, ms_between_reads: u32, condition_closure: C1, execute_closure: C2) -> Result<(), SerialError> 
where
C1: Fn(String) -> bool,
C2: Fn(HashMap<String, String>) -> (),
{
    loop {
        let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
        match parsed_data.get(&operation_key) {
            Some(data) =>  {
                if condition_closure(data.clone()) {
                    execute_closure(parsed_data);
                }
            },
            None => {return Err(SerialError::ErrorInReadValue)}
        }
        FreeRtos::delay_ms(ms_between_reads);
    }
}

pub fn write_when_true(data_reader: &mut (impl READER + WRITER), operation_key: String, ms_between_reads: u32, addr: u8, bytes_to_write: &[u8]) -> Result<(), SerialError> { 
    loop {
        let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
        if parsed_data.contains_key(&operation_key){
            data_reader.parse_and_write(addr, bytes_to_write)?;
        }
        FreeRtos::delay_ms(ms_between_reads);
    }
}

pub fn write_with_frecuency(data_reader: &mut impl WRITER, ms_between_writes: u32, addr: u8, bytes_to_write: &[u8]) -> Result<(), SerialError> {
    loop{
        data_reader.parse_and_write(addr, bytes_to_write)?;
        FreeRtos::delay_ms(ms_between_writes);
    }
}

pub fn micro_to_ticks(time_us: u32) -> u32 {
    configTICK_RATE_HZ * time_us / 1_000_000
}
