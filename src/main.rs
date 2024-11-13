use esp32framework::Microcontroller;

fn main() {
    let mut micro = Microcontroller::take();
    let mut timer_driver = micro.get_timer_driver().unwrap();
    timer_driver.interrupt_after(3_000_000, || println!("Hello World"));
    timer_driver.enable().unwrap();
    println!("WARNING, Hello World incoming...");
    micro.wait_for_updates(None);
}
