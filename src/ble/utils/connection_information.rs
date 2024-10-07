use esp32_nimble::{BLEAddress, BLEConnDesc, BLEError};

/// Contains information about the new client connected that can be user on
/// connection or disconnection callbacks.
///
/// - `address`: The BLEAddress of the remote device to which the server connects.
/// - `id_address`: The public or random identity BLEAddress of the connected client. This address remains constant, even if the client uses private random addresses.
/// - `conn_handle`: A unique `u16` identifier for the current BLE connection. This handle is used internally by the BLE stack to manage connections.
/// - `interval`: A `u16` representing the connection interval, measured in units of 1.25 ms. It determines how frequently data is exchanged between connected devices.
/// - `timeout`: A `u16` representing the connection timeout, measured in units of 10 ms. If no data is received during this time, the connection is considered lost.
/// - `latency`: A `u16` representing the connection latency, indicating the number of connection intervals that can be skipped by the slave if it has no data to send.
/// - `mtu`: A `u16` representing the Maximum Transmission Unit, the maximum size of data that can be sent in a single transmission. This includes the payload plus protocol headers.
/// - `bonded`: A `bool` indicating whether the connection is bonded, meaning the devices have exchanged security keys for secure future connections.
/// - `encrypted`: A `bool` indicating whether the connection is encrypted, meaning the transmitted data is protected against eavesdropping.
/// - `authenticated`: A `bool` indicating whether the connection is authenticated, meaning an authentication process has verified the identity of the devices.
/// - `sec_key_size`: A `u32` representing the size of the security key in bits used in the connection. This affects the security level of the connection.
/// - `rssi`: A `Result<i8, u32>` representing the received signal strength of the connection, measured in dBm. A higher negative value indicates a stronger signal. The result can be a successful `i8` value or an error code `u32`.
/// - `disconnection_result`: A `Option<u32>` representing the disconnection result code, if applicable. It can be `None` if the connection hasn't been disconnected, or a specific error code indicating the cause of the disconnection.
#[derive(Debug, Clone, Copy)]
pub struct ConnectionInformation {
    pub address: BLEAddress,
    pub id_address: BLEAddress,
    pub conn_handle: u16,
    pub interval: u16,
    pub timeout: u16,
    pub latency: u16,
    pub mtu: u16,
    pub bonded: bool,
    pub encrypted: bool,
    pub authenticated: bool,
    pub sec_key_size: u32,
    pub rssi: Result<i8, u32>,
    pub disconnection_result: Option<u32>,
}

impl ConnectionInformation {
    /// Creates a ConnectionInformation from a BLEConnDesc
    ///
    /// # Arguments
    ///
    /// - `server`: The BLEConnDesc that is used to create a ConnectionInformation
    /// - `is_connected`: A boolean to know if the function was called wheter from a disconnection or a connection
    /// - `desc_res`: Result that may contain an error if there was problem that disconnected the client
    ///
    /// # Returns
    ///
    /// A new ConnectionInformation
    pub fn from_bleconn_desc(
        desc: &BLEConnDesc,
        is_connected: bool,
        desc_res: Result<(), BLEError>,
    ) -> Self {
        let mut res = None;
        if !is_connected {
            res = match desc_res {
                Ok(_) => None,
                Err(err) => Some(err.code()),
            };
        }

        let rssi = match desc.get_rssi() {
            Ok(rssi) => Ok(rssi),
            Err(err) => Err(err.code()),
        };

        ConnectionInformation {
            address: desc.address(),
            id_address: desc.id_address(),
            conn_handle: desc.conn_handle(),
            interval: desc.interval(),
            timeout: desc.timeout(),
            latency: desc.latency(),
            mtu: desc.mtu(),
            bonded: desc.bonded(),
            encrypted: desc.encrypted(),
            authenticated: desc.authenticated(),
            sec_key_size: desc.sec_key_size(),
            rssi,
            disconnection_result: res,
        }
    }
}
