use std::{
    sync::{
        atomic::{AtomicBool, AtomicU8, Ordering}, mpsc::{self, Sender}, Arc
    }, u32
};
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::timer;
use esp_idf_svc::hal::peripherals::Peripherals;

static FLAG: AtomicBool = AtomicBool::new(true);

fn main(){
    esp_idf_svc::sys::link_patches();
    let mut timer_driver = timer::TimerDriver::new(unsafe{timer::TIMER00::new()}, &timer::TimerConfig::new()).unwrap();
    let alarm_time  = 1000000 * timer_driver.tick_hz() / 1000000;

    let (tx, rx) = mpsc::channel();
    let t1 = tx.clone();
    let call = move ||{
        let prev = FLAG.load(Ordering::Relaxed);
        FLAG.store(!prev, Ordering::Relaxed);
        t1.send(true);
    };

    unsafe {
        timer_driver.subscribe(call).unwrap()
    }

    timer_driver.enable_interrupt().unwrap();
    timer_driver.enable(true).unwrap();
    timer_driver.set_alarm(15).unwrap();
    timer_driver.enable_alarm(true).unwrap();
    println!("automicbool {}", FLAG.load(Ordering::Relaxed));

    loop{
        let a = rx.recv().unwrap();
        println!("Recibi{}", a);
        let prev = FLAG.load(Ordering::Relaxed);
        println!("automicbool {}", prev);
        let counter = timer_driver.counter().unwrap();
        println!("counter {}", counter);
        FreeRtos::delay_ms(100);
    } 
}

fn callback(s: Sender<bool>){
    let prev = FLAG.load(Ordering::Relaxed);
    FLAG.store(!prev, Ordering::Relaxed);
}
/*
use std::sync::atomic::{AtomicBool, Ordering};

use esp32framework::Microcontroller;
static FLAG: AtomicBool = AtomicBool::new(true);

fn main(){
    let mut micro = Microcontroller::new();
    let mut o1 = micro.set_pin_as_digital_out(2);
    let mut o2 = micro.set_pin_as_digital_out(3);
    let mut o3 = micro.set_pin_as_digital_out(4);
    let mut i1 = micro.set_pin_as_digital_in(5);
    let mut i3 = micro.set_pin_as_digital_in(6);

    o1.blink(10, 4 * 1000000).unwrap();
    o3.blink(10, 8 * 1000000).unwrap();

    let mut i1_old = i1.is_high();
    let mut i3_old = i3.is_high();
    
    loop{
        let i1_new = i1.is_high();
        let i3_new = i3.is_high();
        if i1_new != i1_old && i1_new{
            println!("i1_changed {:?}", i1_new);
        }
        if i3_new != i3_old && i3_new{
            println!("i3 changed {:?}", i3_new);
        }
        
        micro.update(vec![&mut i1, &mut i3], vec![&mut o1, &mut o3]);
        micro.sleep(100);
        i1_old = i1_new;
        i3_old = i3_new;
    }
}

    */
