use esp32framework::{
    Microcontroller
};

fn main(){
    let mut micro = Microcontroller::new();
    let mut timer = micro.get_timer_driver();

    timer.interrupt_after_n_times(1000000, None, true, move || {println!("original")});
    timer.enable().unwrap();
    for i in 0..1000{
        timer.remove_interrupt().unwrap();
        timer.interrupt_after_n_times(1000000, None, true, move || {println!("{i}")});
        timer.enable().unwrap();
    }

    micro.wait_for_updates(None)
}