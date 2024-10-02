//! Example of a ble client using an async approach. The client will connect to a server that has a
//! characteristic of uuid 0x5678. Once connected the client will read all characteristics interpreting
//! their value as an u32 and then multiplies them by a value. This value is obtained from the notifiable
//! characteristics of the service. Thanks to the async aproch we can have other tasks running concurrently
//! to this main function. In this case there is a TimerDriver se to print 'Tic' every 2 seconds.
use esp32_nimble::*;
use esp_idf_svc::hal::{
    delay::FreeRtos,
    prelude::Peripherals,
    task::{asynch::Notification, block_on, queue::Queue},
    timer::{TimerConfig, TimerDriver},
};
use futures::future::join;
use std::sync::Arc;
use utilities::BleUuid;

const SERVICE_UUID: u16 = 0x5678;
const MILLIS: u64 = 2000;
fn main() {
    esp_idf_svc::sys::link_patches();
    let peripherals = Peripherals::take().unwrap();
    let timer0 = peripherals.timer00;
    let timer1 = peripherals.timer10;
    let mut timer_driver0 =
        TimerDriver::new(timer0, &TimerConfig::new().auto_reload(true)).unwrap();
    let timer_driver1 = TimerDriver::new(timer1, &TimerConfig::new()).unwrap();
    let mut client = BLEClient::new();

    let mut characteristics = get_characteristics(&mut client);

    let queue = Arc::new(Queue::new(100));
    set_notify_callback_for_characteristics(&mut characteristics, &queue);
    let print_notification = set_periodical_timer_driver_interrupts(&mut timer_driver0, MILLIS);

    let fut = join(
        main_loop(timer_driver1, characteristics, queue),
        print_tic(print_notification),
    );
    block_on(fut);
}

async fn main_loop<'a>(
    mut timer_driver: TimerDriver<'static>,
    mut characteristics: Vec<&mut BLERemoteCharacteristic>,
    receiver: Arc<Queue<u8>>,
) {
    let mut mult = 2;
    loop {
        for characteristic in characteristics.iter_mut() {
            if !characteristic.can_read() {
                continue;
            }

            let read = characteristic.read_value().await.unwrap();
            let read = get_number_from_bytes(read);
            if let Some((new_mult, _)) = receiver.recv_front(0) {
                mult = new_mult
            }
            let new_value = read.wrapping_mul(mult as u32);
            println!(
                "Characteristic: {:?} Read value: {}, multipling by: {}, result: {}",
                characteristic.uuid(),
                read,
                mult,
                new_value
            );
            if !characteristic.can_write() {
                continue;
            }
            characteristic
                .write_value(&new_value.to_be_bytes(), true)
                .await
                .unwrap();
        }
        timer_driver
            .delay(4 * timer_driver.tick_hz())
            .await
            .unwrap();
    }
}

async fn print_tic(notification: Arc<Notification>) {
    loop {
        notification.wait().await;
        println!("Tic")
    }
}

fn set_periodical_timer_driver_interrupts(
    timer_driver: &mut TimerDriver<'static>,
    millis: u64,
) -> Arc<Notification> {
    let notification = Arc::new(Notification::new());
    let notifier = notification.clone();
    unsafe {
        timer_driver
            .subscribe(move || {
                notifier.notify_lsb();
            })
            .unwrap()
    }

    timer_driver.set_counter(0).unwrap();
    timer_driver.enable_interrupt().unwrap();
    timer_driver
        .set_alarm(millis * timer_driver.tick_hz() / 1000)
        .unwrap();
    timer_driver.enable_alarm(true).unwrap();
    timer_driver.enable(true).unwrap();

    notification
}

fn set_notify_callback_for_characteristics(
    characteristics: &mut Vec<&mut BLERemoteCharacteristic>,
    queue: &Arc<Queue<u8>>,
) {
    for characteristic in characteristics {
        if characteristic.can_notify() {
            let send_q = queue.clone();
            characteristic.on_notify(move |data| {
                //Cannot print due to ISR
                send_q.send_back(data[0], 0).unwrap();
            });
        }
    }
}

fn get_characteristics(client: &mut BLEClient) -> Vec<&mut BLERemoteCharacteristic> {
    block_on(get_characteristics_async(client))
}

async fn get_characteristics_async(client: &mut BLEClient) -> Vec<&mut BLERemoteCharacteristic> {
    let ble_device = BLEDevice::take();
    let ble_scan = ble_device.get_scan();

    println!("Attempting connection");
    let device = find_device(ble_scan).await.unwrap();
    client.connect(device.addr()).await.unwrap();
    println!("Connected");

    FreeRtos::delay_ms(2000);

    let remote_service = client
        .get_service(BleUuid::Uuid16(SERVICE_UUID))
        .await
        .unwrap();

    remote_service
        .get_characteristics()
        .await
        .unwrap()
        .collect()
}

async fn find_device(ble_scan: &mut BLEScan) -> Option<BLEAdvertisedDevice> {
    ble_scan
        .active_scan(true)
        .interval(100)
        .window(99)
        .find_device(i32::MAX, |device| {
            device.is_advertising_service(&BleUuid::Uuid16(SERVICE_UUID))
        })
        .await
        .unwrap()
}

fn get_number_from_bytes(bytes: Vec<u8>) -> u32 {
    let mut aux = vec![0, 0, 0, 0];
    aux.extend(bytes);
    let bytes: [u8; 4] = aux.split_off(aux.len() - 4).as_slice().try_into().unwrap();
    u32::from_be_bytes(bytes)
}
