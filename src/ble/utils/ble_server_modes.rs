use esp32_nimble::enums::{ConnMode, DiscMode};

/// Enums the posible discoverable modes:
/// * `Non-Discoverable Mode`: The device does not advertise itself. Other devices will connect only if they know the specific address.
/// * `General Discoverable Mode`: The advertisment is done continuously, so any other device can see it in any moment.
///
/// Both Limited and General Discoverable Mode have min_interval and max_interval:
/// * `min_interval`: The minimum advertising interval, time between advertisememts. This value
///   must range between 20ms and 10240ms in 0.625ms units.
/// * `max_interval`: The maximum advertising intervaltime between advertisememts. TThis value
///   must range between 20ms and 10240ms in 0.625ms units.
#[derive(Debug)]
pub enum DiscoverableMode {
    GeneralDiscoverable(u16, u16),
    NonDiscoverable,
}

impl DiscoverableMode {
    /// Gets the DiscMode from a DiscoverableMode
    ///
    /// # Returns
    ///
    /// The corresponding DiscMode
    pub fn get_code(&self) -> DiscMode {
        match self {
            DiscoverableMode::NonDiscoverable => DiscMode::Non,
            DiscoverableMode::GeneralDiscoverable(_, _) => DiscMode::Gen,
        }
    }
}

/// Enums the posible connection modes:
/// * `NonConnectable`: The device does not allow connections.
/// * `UndirectedConnectable`: The divice allows connections from any device.
#[derive(Debug)]
pub enum ConnectionMode {
    NonConnectable,
    UndirectedConnectable,
}

impl ConnectionMode {
    /// Gets the ConnMode from a ConnectionMode
    ///
    /// # Returns
    ///
    /// The corresponding ConnMode
    pub fn get_code(&self) -> ConnMode {
        match self {
            ConnectionMode::NonConnectable => ConnMode::Non,
            ConnectionMode::UndirectedConnectable => ConnMode::Und,
        }
    }
}
