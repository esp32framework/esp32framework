# Esp32framework

A Esp32 Framework for developing IoT applications in a straightforward manner, requiring minimal bare-metal technical knowledge. This Framework adds further abstraction layers over the Esp-Idf-Svc(link) Rust crate, which hide some of the complexity to give a more beginner-friendly API, and introducing some extra common functionalities.

## Scope
In this section, we specify the type of projects that are compatible with our framework. This takes into considerations technical specifications such as the protocols used and the overall project context.

### Who is this for
This Framework is designed for anyone looking to create a project with a high level of abstraction. These pre-build abstractions facilitate the rapid use of multiple protocols with minimum technical knowledge about it. Depending on the project, this can save multiples hours that would otherwise be spent reading technical documentation, such as datasheets.  
However, this framework does not aim to optimize microcontroller resources. Therefore, projects that rely on memory usage optimizations, low power consumtion, or are extremely time sensitive may not be suitable for development within this framework.

### Protocols & Technologies
- GPIO: 
    - Digital in
    - Digital out
    - Analogic in using built in ADC (Analogical to Digital Converter)
    - Analogic in using PWM (Pulse Width Modulation) signals
    - Analogic out using PWM (Pulse Width Modulation) signals 

- TimerDriver: (Driver for timer resource, allows for multiple interrupts per timer)

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
    - HC-SR04 (Ultrasonic Distance Sensor)
    - DS3231 (Real-Time Clock & Temperature)

Each technology comes with its own set of examples that demonstrate basic configurations and common use cases.   
This list of supported Protocols and Technologies is continuously growing, and we encourage users to create their own abstractions for new protocols or sensors to contribute to the framework.

## SetUp

### Prerequisits
Before generating a new proyect make sure you meet the following requirements.

#### Install rust with rustup
Make sure to install rust version 1.77 or greater using rustup, since it will be necesary for installing further dependencies.

#### Install Cargo Sub-Comands
To install the cargo sub commands run:
```sh
cargo install cargo-generate
cargo install ldproxy
cargo install espup
cargo install espflash
cargo install cargo-espflash # Optional
```

You may be missing some dependencies needed for the cargo subcommands which can be installed with the following.

```sh
apt-get install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0
apt install pkg-config
apt-get install libudev-dev 
```

#### Install Rust & Clang toolchains for Espressif SoCs (with espup)
```sh
espup install
```

if command is not found you may need to look for espup bin inside the cargo bin folder.

```sh
~/.cargo/bin/espup install # This is the cargo bin default location
```

For a more detailed explanation see [Setting Up a Development Environment](https://docs.esp-rs.org/book/installation/index.html)  chapter of The Rust on ESP Book.
### Generate the proyect
Generate the proyect by runninng
```sh
cargo generate esp32framework/esp32framework-template
```
This template was based upon the Rust on the [ESP-IDF "Hello, World" template](https://github.com/esp-rs/esp-idf-template?tab=readme-ov-file) with some modifications

## Execute
To execute your proyect you can simply run
`cargo run`. This will build and flash the proyect and will leave you with a serial monitor.

You may also do this by running each step separatly.
```sh
cargo build
espflash flash target/riscv32imac-esp-espidf/debug/esp32framework
espflash monitor
```

## About us
We are a team of four developers who designed this Framework in 2024 as our Final Proyect for the Software Engineering degree at Universidad de Buenos Aires. Our profiles are:  
[DiegoC](https://github.com/DiegoCivi)  
[Joaquin](https://github.com/Rivejjj)  
[Juan Pablo Aschieri](https://github.com/higlak)  
[mateogdupont](https://github.com/mateogdupont)  

## Disclaimers

This project is still in the relatively early stages of development, and as such there should be no expectation of API stability. A significant number of peripherals currently have drivers implemented but have varying levels of functionality. For most tasks, this should be usable already, however some more advanced or uncommon features may not yet be implemented.
