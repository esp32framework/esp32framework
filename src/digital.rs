
struct Pin {
    number: u8,
    protocols: Vector<>,
}

struct DigitalOut{
    pin: Pin,
    value: DigitalValue
}

struct DigitalIn{
    pin: Pin,
    react_to: Flank,
    debounce_time: u32, //cantidad de microsegundos
    read_interval: u32
}

enum Flank {
    Ascending,
    Descending,
    Both,
}

#[derive(Eq, PartialEq)]
enum DigitalValue {
    High,
    Low
}

impl DigitalOut{
    fn new() -> DigitalOut {
        
    }
    
    fn is_high(self) -> bool{
        return self.value == DigitalValue::High
    }

    fn is_low(self) -> bool{
        return self.value == DigitalValue::Low
    }
    
    fn set_high(self) -> Result<>{
        self.value = DigitalValue::High
    }
    
    fn set_low(self) -> Result<>{
        self.value = DigitalValue::Low
    }
    
    fn toggle(self) -> Result<>{
        //self.value = 
    }
    
    fn blink(self, frequency, duration) -> Result<>{
        
    }
}

impl DigitalIn {
    fn new(flank: Flank) -> DigitalIn {
        
    }

    fn trigger_on_flank(self, func){
        
    }

    fn trigger_on_flank_first_n_times(self, amount_of_times: usize, func){
    
    }
    
    fn get_value() -> DigitalValue{
        
    }    

    fn is_high(self) -> bool{
        self.get_value() == DigitalValue::High
    }
    
    fn is_low(self) -> bool{
        self.get_value() == DigitalValue::Low 
    }
    
    fn set_debounce(self){

    }

    fn set_read_intervals(self){
        
    }
}