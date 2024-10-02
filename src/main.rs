use esp32framework::Microcontroller;

fn main() {
    let mut micro = Microcontroller::new();
    let mut timer = micro.get_timer_driver().unwrap();
    timer.interrupt_after(2000000, || println!("el timer esta bien"));
    timer.enable().unwrap();

    micro.wait_for_updates(None).unwrap();
}
