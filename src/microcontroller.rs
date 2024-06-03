use esp_idf_svc::hal::peripherals::Peripherals;


struct Microcontroller{
    peripherals: Peripherals
}

impl Microcontroller{
    fn new() -> Self{
        Microcontroller{
            peripherals: Peripherals::take().unwrap(),
        }
    }

    fn set_pin_as_digital_out(pin: u32) {
        let pin = _get_pin(pin);
        let mut digital_out = PinDriver::output(self.peripherals.pins.pin);
        digital_out(pin)
    }

    fn set_pin_as_digital_in(pin: u32) {
        let pin = _get_pin(pin);
        let mut digital_in = PinDriver::input(self.peripherals.pins.pin);
        digital_in
    }
    
    fn run<F: Fn>(self, ){
        loop{
            
        }
    }
}