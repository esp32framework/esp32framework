struct AnalogOut {
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
