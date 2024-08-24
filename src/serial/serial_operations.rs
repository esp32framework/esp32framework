use std::collections::HashMap;

use esp_idf_svc::hal::delay::FreeRtos;

pub enum SerialError {
    ErrorInReadValue,
}

/// Trait for performing reading and parsing operations.
pub trait READER {
    /// Reads data and parses it into a `HashMap<String, String>`.
    fn read_and_parse(& mut self) -> HashMap<String, String>;
}

/// Trait for performing writing and parsing operations.
pub trait WRITER { 
    /// Parses and writes data to a specific address.
    fn parse_and_write(&mut self, addr: u8, bytes_to_write: &[u8]) -> Result<(), SerialError>;
}

/// Reads and prints the value specified
pub fn show_data(data_reader: &mut impl READER, operation_key: String) -> Result<(), SerialError> {
    let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
    match parsed_data.get(&operation_key) {
        Some(data) => println!("The content is: {:?}", data),
        None => {return Err(SerialError::ErrorInReadValue);}
    }
    Ok(())
}

/// Reads "times" and returns the total sum of the values as an `f32` on success. Returns an error if the key is not found
/// or if a value cannot be parsed to `f32`.
/// Note: Float values must use a dot as the decimal separator. Values with a comma are not supported.
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

/// Reads "times" and returns the average of the values as an `f32` on success. Returns an error if the key is not found
/// or if a value cannot be parsed to `f32`.
/// 
/// Note: Float values must use a dot as the decimal separator. Values with a comma are not supported.
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

/// Reads "times",Collects the values as a `Vec<String>` and applies the provided closure to this vector. Returns the result
/// of the closure on success. Returns an error if the key is not found or if any value cannot be read.
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

/// Loops indefinitely, checking the condition for each read. If the condition is met, `execute_closure`
/// is called with the parsed data. Returns an error if the key is not found or if there is an issue reading the data.
/// 
/// Note: The function runs indefinitely, ensure that `condition_closure` eventually returns `true`.
pub fn execute_when_true<C1, C2>(data_reader: &mut impl READER, operation_key: String, ms_between_reads: u32, condition_closure: C1, execute_closure: C2) -> Result<(), SerialError> 
where
C1: Fn(String) -> bool,
C2: Fn(HashMap<String, String>),
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

/// The function loops indefinitely, writting each time `operation_key` is found in the parsed data .Returns an error 
/// if the write operation fails or if the data cannot be read.
/// 
/// Note: The function runs indefinitely, ensure that `condition_closure` eventually returns `true`.
pub fn write_when_true(data_reader: &mut (impl READER + WRITER), operation_key: String, ms_between_reads: u32, addr: u8, bytes_to_write: &[u8]) -> Result<(), SerialError> { 
    loop {
        let parsed_data: HashMap<String, String> = data_reader.read_and_parse();
        if parsed_data.contains_key(&operation_key){
            data_reader.parse_and_write(addr, bytes_to_write)?;
        }
        FreeRtos::delay_ms(ms_between_reads);
    }
}

/// The function loops indefinitely, performing the write operation at the specified frequency. Returns an error if the
/// write operation fails.
pub fn write_with_frecuency(data_reader: &mut impl WRITER, ms_between_writes: u32, addr: u8, bytes_to_write: &[u8]) -> Result<(), SerialError> {
    loop{
        data_reader.parse_and_write(addr, bytes_to_write)?;
        FreeRtos::delay_ms(ms_between_writes);
    }
}


