## Esp32framework

A Esp32 Framework for developing IOT applications in a simple way, with minimun bare metal technical knowledge. This adds further abstraction layers over the Esp-Idf-Svc(link) rust crate, which hide some of the complexity to give a more beginner friendly api, and add some extra common functionalities.

### Scope

#### Who is this for
 

#### Protocols & Technologies
- GPIO: 
    - Digital in
    - Digital out
    - Analogic in using built in ADC (Analogical to Digital Converter)
    - Analogic in using PWM (Pulse Width Modulation) signals
    - Analogic out using PWM (Pulse Width Modulation) signals 

- TimerDriver:

- Serial:
    - I2C
    - UART

- BLE(Bluetooth Low Energy):
    - Ble Beacon
    - Ble Server
    - Ble Client

- WIFI:
    - Http client
    - Https client

- Sensors:
    - HC-SR04
    - DS3231

### SetUp

### Execute

### Que perdimos

### About us

### Disclaimers

This project is still in the relatively early stages of development, and as such there should be no expectation of API stability. A significant number of peripherals currently have drivers implemented but have varying levels of functionality. For most tasks, this should be usable already, however some more advanced or uncommon features may not yet be implemented.
