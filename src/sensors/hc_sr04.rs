use std::{num::NonZero, sync::{atomic::AtomicU32, Arc}};
use esp_idf_svc::{hal::{delay::Delay, gpio::InterruptType, task::notification::{self, Notifier}}, sys::esp_timer_get_time};
use crate::gpio::{DigitalIn, DigitalOut};
use esp_idf_svc::hal::task::notification::Notification;


const SOUND_SPEED_M_S: f64 = 340.0;
const SOUND_SPEED_CM_US: f64 = SOUND_SPEED_M_S * 100.0 / 1_000_000.0;


pub struct HCSR04<'a> {
    trig: DigitalOut<'a>,
    echo: DigitalIn<'a>,
    echo_ans_time: Arc<AtomicU32> // TODO: We actually need an AtomicU64, but there are problems with the import
}

impl <'a>HCSR04<'a> {

    pub fn new(trig: DigitalOut<'a>, echo: DigitalIn<'a>) -> HCSR04<'a> {
        let echo_ans_time = Arc::new(AtomicU32::new(0));
        HCSR04 { trig, echo, echo_ans_time }
    }

    pub fn get_distance(&mut self) -> f64 { // TODO: The polling on the whiles dont have sleeps. Need a way to use interrupts or other mechanism to release the CPU
        let delay = Delay::new_default();
        let notification = Notification::new();
        let notifier = notification.notifier();
        /*
        // Callback to notify when the echo has been received
        let callback = move || {
            unsafe { notifier.notify_and_yield(NonZero::new(1).unwrap()) };
        };*/
        // First set the trigger to Low for a few micro-seconds to get a clean signal
        // Then set the trigger pin high for 10 micro-seconds to send the sonic burst
        self.trig.set_low().unwrap();
        delay.delay_us(4);
        self.trig.set_high().unwrap();
        delay.delay_us(10);
        self.trig.set_low().unwrap();
    
        
        while self.echo.is_low() {
            // Nothing
        }
        let send_echo_time = unsafe { esp_timer_get_time() };
        /*
        println!("aaaaaaaaaa");
        self.echo.trigger_on_interrupt(callback, InterruptType::NegEdge).unwrap();
        notification.wait_any();
        println!("{:?}", self.echo.get_level());
        */
        while self.echo.is_high() {
            // Nothing
        }
        let rec_echo_time = unsafe { esp_timer_get_time() };
        
        let travel_time = rec_echo_time - send_echo_time;
        let cm: f64 = SOUND_SPEED_CM_US * travel_time as f64;
        cm / 2.0 // We divide by 2 because if not we get the distance of the roundtrip
    }
}
