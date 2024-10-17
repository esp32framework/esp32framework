use super::BleError;
use esp32_nimble::enums::{AuthReq, SecurityIOCap};

const MAX_PASKEY: u32 = 999999;

/// Enums the device's input and output capabilities,
/// which help determine the level of security and the key
/// generation method for pairing:
/// - `DisplayOnly`: It is capable of displaying information on a
///    screen but cannot receive inputs.
/// - `DisplayYesNo`: It can display information and/or yes/no questions,
///    allowing for limited interaction.
/// - `KeyboardOnly`: It can receive input through a keyboard
///    (e.g., entering a PIN during pairing).
/// - `NoInputNoOutput`: It has no means to display information or
///    receive input from, for example, keyboards or buttons.
/// - `KeyboardDisplay`: It can receive input through a keyboard and it
///    is capable of displaying information.
pub enum IOCapabilities {
    DisplayOnly,
    DisplayYesNo,
    KeyboardOnly,
    KeyboardDisplay,
    NoInputNoOutput,
}

impl IOCapabilities {
    /// Gets the corresponding SecurityIOCap
    ///
    /// # Returns
    ///
    /// A SecurityIOCap
    pub fn get_code(&self) -> SecurityIOCap {
        match self {
            IOCapabilities::DisplayOnly => SecurityIOCap::DisplayOnly,
            IOCapabilities::DisplayYesNo => SecurityIOCap::DisplayYesNo,
            IOCapabilities::KeyboardOnly => SecurityIOCap::KeyboardOnly,
            IOCapabilities::NoInputNoOutput => SecurityIOCap::NoInputNoOutput,
            IOCapabilities::KeyboardDisplay => SecurityIOCap::KeyboardDisplay,
        }
    }
}
/// Contains the necessary to have a secure BLE server.
/// This includes a passkey, the I/O capabilities and the
/// authorization requirements.
/// - `passkey`: A 6-digit u32
/// - `auth_mode`: An u8 representing the combination of authorization modes
/// - `io_capabilities`: An IOCapabilities instance
pub struct Security {
    pub(crate) passkey: u32,
    pub(crate) auth_mode: u8,
    pub(crate) io_capabilities: IOCapabilities,
}

impl Security {
    /// Creates a Security with its passkey of a maximum of 6 digits and I/O capabilities.
    ///
    /// It has no authentication requirements, this need to be set separately
    ///
    /// # Arguments
    ///
    /// - `passkey`: A 6-digit u32
    /// - `io_capabilities`: An IOCapabilities instance
    ///
    /// # Returns
    ///
    /// A new Security instance
    pub fn new(passkey: u32, io_capabilities: IOCapabilities) -> Result<Self, BleError> {
        if passkey > MAX_PASKEY {
            return Err(BleError::InvalidPasskey);
        }
        Ok(Security {
            passkey,
            auth_mode: 0,
            io_capabilities,
        })
    }

    /// Adds or removes a authorization requirement to the security instance
    ///
    /// # Arguments
    ///
    /// - `value`: A bool. When True the requirement is added. When False the requirement is removed
    /// - `flag`: The AuthReq to add or remove
    ///
    /// # Returns
    ///
    /// The Security itself
    fn toggle(&mut self, value: bool, flag: AuthReq) -> &mut Self {
        if value {
            self.auth_mode |= flag.bits();
        } else {
            self.auth_mode &= !flag.bits();
        }
        self
    }

    /// Sets the Allow Bonding authorization requirement.
    ///
    /// When the bonding is allowed, devices remember the
    /// pairing information. This allows to make future conexions to be faster
    /// and more secure. Useful for devices that get connected with frequency.
    ///
    /// # Arguments
    ///
    /// - `value`: A bool. When True the requirement is added. When False the requirement is removed
    ///
    /// # Returns
    ///
    /// The Security itself
    pub fn allow_bonding(&mut self, value: bool) -> &mut Self {
        self.toggle(value, AuthReq::Bond);
        self
    }

    /// Sets the Man in the Middle authorization requirement.
    ///
    /// Authentication requires a verification
    /// that makes it hard for a third party to intercept the communication.
    ///
    /// # Arguments
    ///
    /// - `value`: A bool. When True the requirement is added. When False the requirement is removed
    ///
    /// # Returns
    ///
    /// The Security itself
    pub fn man_in_the_middle(&mut self, value: bool) -> &mut Self {
        self.toggle(value, AuthReq::Mitm);
        self
    }

    /// Sets the Secure Connection authorization requirement.
    ///
    /// This is a more secure version of BLE pairing by using the
    /// elliptic curve Diffie-Hellman algorithm. This is part of standard Bluetooth 4.2 and newer versions.
    ///
    /// # Arguments
    ///
    /// - `value`: A bool. When True the requirement is added. When False the requirement is removed
    ///
    /// # Returns
    ///
    /// The Security itself
    pub fn secure_connection(&mut self, value: bool) -> &mut Self {
        self.toggle(value, AuthReq::Sc);
        self
    }
}
