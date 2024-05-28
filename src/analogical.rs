struct AnalogOut {
    pin: Pin,
    resolution: usize, // cantidad de bytes de representacion del voltaje
    frequency: usize,
    
}

struct AnalogIn {
    pin: Pin,
    resolution: usize, // cantidad de bytes de representacion del voltaje
    frequency: usize,
}

enum AnalogicTriggerCondition{
    InRange(u32,u32),
    OutOfRange(u32,u32),
    HigherThan(u32),
    LowerThan(u32)
}

impl AnalogOut {
    fn new() -> AnalogOut {

    }

    fn set_value(value: usize){
        //value < 2**resolution
    }

    fn set_frequency(self){

    }

    fn set_resolution(self){
        
    }
}

impl AnalogIn {
    fn new() -> AnalogIn {
        
    }

    fn trigger_when(self, condition :AnalogicTriggerCondition, func){
    
    }

    fn trigger_first_n_times(self, condition :AnalogicTriggerCondition, amount_of_times: usize, func){
    
    }

    fn read(self)->usize{
        
    }

    fn set_frequency(self){

    }

    fn set_resolution(self){
        
    }
}

