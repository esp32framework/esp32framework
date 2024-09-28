//! Example of a ble client using an async approach. The client will connect to a server that has a 
//! characteristic of uuid 0x12345678. Once connected the client will read all characteristics interpreting
//! their value as an u32 and then multiplies them by a value. This value is obtained from the notifiable 
//! characteristics of the service. Thanks to the async aproch we can have other tasks running concurrently
//! to this main function. In this case there is a TimerDriver se to print 'Tic' every 2 seconds.
use std::{sync::Arc, time::Duration};
use esp32_nimble::*;
use esp32framework::Microcontroller;
use esp_idf_svc::hal::{delay::FreeRtos, prelude::Peripherals, task::{asynch::Notification, block_on, queue::Queue}, timer::{TimerConfig, TimerDriver}};
use futures::future::join;
use utilities::BleUuid;

const SERVICE_UUID: u32 = 0x12345678;

fn main(){
	//esp_idf_svc::sys::link_patches();
    //block_on(main_loop())

    let mut micro = Microcontroller::new();
    let mut client = micro.ble_client();
    let device = client.find_device_with_service(None, &esp32framework::ble::BleId::FromUuid32(SERVICE_UUID)).unwrap();

    client.connect_to_device(device).unwrap();
    micro.wait_for_updates(None);
}