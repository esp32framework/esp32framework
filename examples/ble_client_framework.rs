//! Example of a ble client using an async aproach. The client will connect to a server that has a
//! service of uuid 0x5678. Once connected the client will read all characteristics interpreting
//! their value as an u32 and then multiplies them by a value. This value is obtained from the notifiable
//! characteristics of the service. Thanks to the async aproch we can have other tasks running concurrently
//! to this main function. In this case there is a TimerDriver se to print 'Tic' every 2 seconds.

use std::sync::mpsc::{self, Receiver};

use esp32framework::{
    ble::{utils::RemoteCharacteristic, BleError, BleId},
    timer_driver::TimerDriver,
    Microcontroller,
};

fn main() {
    let mut micro = Microcontroller::new();

    let mut characteristics = get_characteristics(&mut micro);

    let receiver = set_notify_callback_for_characteristics(&mut characteristics);
    let timer_driver = set_periodical_timer_driver_interrupts(&mut micro, 2000);

    micro
        .block_on(main_loop(timer_driver, characteristics, receiver))
        .unwrap();
}

fn get_characteristics(micro: &mut Microcontroller) -> Vec<RemoteCharacteristic> {
    let mut client = micro.ble_client().unwrap();
    let service_id = BleId::FromUuid16(0x5678);
    println!("Attempting connection");

    let device = client.find_device_with_service(None, &service_id).unwrap();
    client.connect_to_device(device).unwrap();

    println!("Connected");
    micro.wait_for_updates(Some(2000)).unwrap();

    client.get_all_characteristics(&service_id).unwrap()
}

fn set_notify_callback_for_characteristics(
    characteristics: &mut Vec<RemoteCharacteristic>,
) -> Receiver<u8> {
    let (sender, receiver) = mpsc::channel();

    for characteristic in characteristics {
        let s = sender.clone();
        _ = characteristic.on_notify(move |data| {
            println!("Received_notif mult {}", data[0]);
            s.send(data[0]).unwrap();
        });
    }

    receiver
}

fn set_periodical_timer_driver_interrupts<'a>(
    micro: &mut Microcontroller<'a>,
    mili: u64,
) -> TimerDriver<'a> {
    let mut timer_driver = micro.get_timer_driver().unwrap();
    timer_driver.interrupt_after_n_times(mili * 1000, None, true, || println!("Tic"));
    timer_driver.enable().unwrap();
    timer_driver
}

async fn main_loop(
    mut timer_driver: TimerDriver<'_>,
    mut characteristics: Vec<RemoteCharacteristic>,
    receiver: Receiver<u8>,
) {
    let mut mult = 2;
    loop {
        for characteristic in characteristics.iter_mut() {
            let read = match characteristic.read_async().await {
                Ok(read) => get_number_from_bytes(read),
                Err(err) => match err {
                    BleError::CharacteristicNotReadable => continue,
                    _ => panic!("{:?}", err),
                },
            };

            if let Ok(new_mult) = receiver.try_recv() {
                mult = new_mult
            }
            let new_value = read.wrapping_mul(mult as u32);

            println!(
                "Characteristic: {:?} Read value: {}, multipling by: {}, result: {}",
                characteristic.id(),
                read,
                mult,
                new_value
            );

            if let Err(err) = characteristic.write_async(&new_value.to_be_bytes()).await {
                match err {
                    BleError::CharacteristicNotWritable => continue,
                    _ => panic!("{:?}", err),
                }
            }
        }

        timer_driver.delay(4000).await.unwrap();
    }
}

fn get_number_from_bytes(bytes: Vec<u8>) -> u32 {
    let mut aux = vec![0, 0, 0, 0];
    aux.extend(bytes);
    let bytes: [u8; 4] = aux.split_off(aux.len() - 4).as_slice().try_into().unwrap();
    u32::from_be_bytes(bytes)
}
