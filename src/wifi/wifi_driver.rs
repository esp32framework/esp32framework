use crate::microcontroller_src::peripherals::PeripheralError;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        modem::{self},
        task::block_on,
    },
    nvs::EspDefaultNvsPartition,
    sys::ESP_ERR_TIMEOUT,
    timer::EspTaskTimerService,
    wifi::{AccessPointInfo, AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use std::{net::Ipv4Addr, time::Duration};

use super::http::{Http, HttpClient, HttpsClient};

/// Error types related to WIFI operations.
#[derive(Debug)]
pub enum WifiError {
    ConfigurationError,
    ConnectingError,
    ConnectionTimeout,
    DnsNotFound,
    HttpError,
    InformationError,
    NvsAlreadyTaken,
    PeripheralError(PeripheralError),
    StartingError,
    WifiNotInitialized,
    ScanError,
}

/// Abstraction of an Acces Point with its basic information.
pub struct AccesPoint {
    pub ssid: String,
    pub authentication_method: String,
    pub signal_strength: i8,
}

impl From<AccessPointInfo> for AccesPoint {
    fn from(value: AccessPointInfo) -> Self {
        AccesPoint {
            ssid: value.ssid.to_string(),
            authentication_method: match value.auth_method {
                Some(AuthMethod::WEP) => String::from("WEP"),
                Some(AuthMethod::WPA) => String::from("WPA"),
                Some(AuthMethod::WPA2Personal) => String::from("WPA2-Personal"),
                Some(AuthMethod::WPAWPA2Personal) => String::from("WPA/WPA2-Personal"),
                Some(AuthMethod::WPA2Enterprise) => String::from("WPA2-Enterprise"),
                Some(AuthMethod::WPA3Personal) => String::from("WPA3-Personal"),
                Some(AuthMethod::WPA2WPA3Personal) => String::from("WPA2/WPA3-Personal"),
                Some(AuthMethod::WAPIPersonal) => String::from("WAPI"),
                _ => String::from("None"),
            },
            signal_strength: value.signal_strength,
        }
    }
}

/// Abstraction of the driver that controls the wifi. It simplifies
/// the wifi connection and the creation of an HTTP client.
pub struct WifiDriver<'a> {
    controller: AsyncWifi<EspWifi<'a>>,
}

impl<'a> WifiDriver<'a> {
    /// Creates a new WifiDriver.
    ///
    /// By default this function takes the Non-Volatile Storage of the ESP in order to save
    /// wifi configuration. This is to improve connection times for future connections
    /// to the same network.
    ///
    /// # Arguments
    ///
    /// - `event_loop`: Microcontroller's event loop.
    /// - `modem`: Microcontroller's modem peripheral.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `WifiDriver` instance, or an `WifiError` if the
    /// initialization fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::NvsAlreadyTaken`: If the NVS Default Partition was already taken.
    /// - `WifiError::StartingError`: If there is an error initializing the driver.
    pub(crate) fn new(
        event_loop: EspSystemEventLoop,
        modem: modem::Modem,
    ) -> Result<Self, WifiError> {
        let nvs = EspDefaultNvsPartition::take().map_err(|_| WifiError::NvsAlreadyTaken)?;
        let timer_service = EspTaskTimerService::new().map_err(|_| WifiError::StartingError)?;
        Ok(WifiDriver {
            controller: AsyncWifi::wrap(
                EspWifi::new(modem, event_loop.clone(), Some(nvs))
                    .map_err(|_| WifiError::StartingError)?,
                event_loop,
                timer_service,
            )
            .map_err(|_| WifiError::StartingError)?,
        })
    }

    /// Attempts a connection to the desired wifi network.
    ///
    /// If a password is passed, it connects using the WPAWPA2Personal Authentication method.
    /// Otherwise, it doesn't use an Authentication method.
    /// If a timeout is passed it will timeout after attempting a connection for that time
    ///
    /// # Arguments
    ///
    /// - `ssid`: A &str representing the SSID to connect to.
    /// - `password`: An `Option<String>` that may contain the password of the SSID.
    /// - `timeout`: An `Option<Duration>` that may contain the dessired timeout
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the connection completed successfully, or an `WifiError` if it fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::ConfigurationError`: If the configuration of the wifi driver fails.
    /// - `WifiError::StartingError`: Error while starting wifi driver.
    /// - `WifiError::ConnectingError`: Error while connecting to wifi.
    /// - `WifiError::ConnectionTimeout`: TimedOut while trying to connect.
    pub fn connect(
        &mut self,
        ssid: &str,
        password: Option<String>,
        timeout: Option<Duration>,
    ) -> Result<(), WifiError> {
        block_on(self.connect_async(ssid, password, timeout))
    }

    /// Async version of [Self::connect]
    pub async fn connect_async(
        &mut self,
        ssid: &str,
        password: Option<String>,
        timeout: Option<Duration>,
    ) -> Result<(), WifiError> {
        self.set_connection_configuration(ssid, password)?;

        self.controller
            .start()
            .await
            .map_err(|_| WifiError::StartingError)?;

        self._connect(timeout).await
    }

    /// Sets the necessary configurations to attempt a connection
    ///
    /// If a password is passed, it connects using the WPAWPA2Personal Authentication method.
    /// Otherwise, it doesn't use an Authentication method.
    ///
    /// # Arguments
    ///
    /// - `ssid`: A &str representing the SSID to connect to.
    /// - `password`: An `Option<String>` that may contain the password of the SSID.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the configuration completed successfully, or an `WifiError` if it fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::ConfigurationError`: If the configuration of the wifi driver fails.
    fn set_connection_configuration(
        &mut self,
        ssid: &str,
        password: Option<String>,
    ) -> Result<(), WifiError> {
        let auth_method = match password {
            Some(_) => AuthMethod::WPAWPA2Personal,
            None => AuthMethod::None,
        };
        let wifi_pass = password.unwrap_or("".to_string());

        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: ssid.try_into().map_err(|_| WifiError::ConfigurationError)?,
            bssid: None, // MAC address
            auth_method,
            password: (wifi_pass.as_str())
                .try_into()
                .map_err(|_| WifiError::ConfigurationError)?,
            channel: None,
            ..Default::default()
        });

        self.controller
            .set_configuration(&wifi_configuration)
            .map_err(|_| WifiError::ConfigurationError)
    }

    /// Attempts a connection to the desired wifi network, assuming that the configuration has already been set
    ///
    /// If a timeout is passed it will timeout after attempting a connection for that time
    ///
    /// # Arguments
    ///
    /// - `timeout`: An `Option<Duration>` that may contain the dessired timeout
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the connection completed successfully, or an `WifiError` if it fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::ConnectingError`: Error while connecting to wifi.
    /// - `WifiError::ConnectionTimeout`: TimedOut while trying to connect.
    async fn _connect(&mut self, timeout: Option<Duration>) -> Result<(), WifiError> {
        self.controller
            .wifi_mut()
            .connect()
            .map_err(|_| WifiError::ConnectingError)?;
        self.controller
            .wifi_wait(|this| this.wifi().is_connected().map(|s| !s), timeout)
            .await
            .map_err(|err| match err.code() {
                ESP_ERR_TIMEOUT => WifiError::ConnectionTimeout,
                _ => WifiError::ConnectingError,
            })?;

        self.controller
            .wait_netif_up()
            .await
            .map_err(|_| WifiError::ConnectingError)?;
        Ok(())
    }

    /// Scans for nearby Wi-Fi networks and returns a vector of discovered access points.
    ///     
    ///  # Returns
    ///
    /// A Result containing a vector of the discovered access points. Else, a `WifiError`.
    ///
    /// # Errors
    ///
    /// - `WifiError::ScanError`: If the scan operation fails to complete successfully.
    /// - `WifiError::StartingError`: Error while starting wifi driver.
    pub fn scan(&mut self) -> Result<Vec<AccesPoint>, WifiError> {
        block_on(self.scan_async())
    }

    /// Async version of [Self::scan]
    pub async fn scan_async(&mut self) -> Result<Vec<AccesPoint>, WifiError> {
        if !self.is_started() {
            self.controller
                .start()
                .await
                .map_err(|_| WifiError::StartingError)?;
        }

        let results: Vec<AccessPointInfo> = self
            .controller
            .scan()
            .await
            .map_err(|_| WifiError::ScanError)?;

        let parsed_results: Vec<AccesPoint> = results.into_iter().map(AccesPoint::from).collect();

        Ok(parsed_results)
    }
    /// Checks if the driver is already started.
    ///
    /// # Returns
    ///
    /// A bool that indicated whether the driver has started or not.
    pub fn is_started(&self) -> bool {
        self.controller.is_started().unwrap_or(false)
    }

    /// Checks if the driver is already connectedto a wifi network.
    ///
    /// # Returns
    ///
    /// A Result containing a bool that indicated whether the driver has connected or not. Else,
    /// a `WifiError`.
    ///
    /// # Errors
    ///
    /// - `WifiError::WifiNotInitialized`: If WiFi is not initialized by esp_wifi_init.
    pub fn is_connected(&self) -> Result<bool, WifiError> {
        self.controller
            .is_connected()
            .map_err(|_| WifiError::WifiNotInitialized)
    }

    /// Get the ip address of the device.
    ///
    /// # Returns
    ///
    /// A Result containing a Ipv4Addr with the device ip address or a `WifiError` in case of failure.
    ///
    /// # Errors
    ///
    /// - `WifiError::InformationError`: If WiFi driver can not get its own ip.
    pub fn get_address_info(&self) -> Result<Ipv4Addr, WifiError> {
        let netif = self.controller.wifi().sta_netif();
        let info = netif
            .get_ip_info()
            .map_err(|_| WifiError::InformationError)?;
        Ok(info.ip)
    }

    /// Gets the DNS ip address.
    ///
    /// # Returns
    ///
    /// A Results containing an Ipv4Addr or a `WifiError` if if fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::InformationError`: If getting the informatiof of the netif fails.
    /// - `WifiError::DnsNotFound`: If the netif info does not have the dns ip address.
    pub fn get_dns_info(&self) -> Result<Ipv4Addr, WifiError> {
        let netif = self.controller.wifi().sta_netif();
        let info = netif
            .get_ip_info()
            .map_err(|_| WifiError::InformationError)?;
        match info.dns {
            Some(ip) => Ok(ip),
            None => Err(WifiError::DnsNotFound),
        }
    }

    /// Creates a new HttpClient ready to use.
    ///
    /// # Returns
    ///
    /// A Result containing the new HttpClient or a `WifiError` if the inizialization fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::HttpError`: If the inizialization of the HttpClient fails.
    pub fn get_http_client(&self) -> Result<HttpClient, WifiError> {
        HttpClient::new().map_err(|_| WifiError::HttpError)
    }

    /// Creates a new HttpsClient ready to use.
    ///
    /// # Returns
    ///
    /// A Result containing the new HttpsClient or a `WifiError` if the inizialization fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::HttpError`: If the inizialization of the HttpsClient fails.
    pub fn get_https_client(&self) -> Result<HttpsClient, WifiError> {
        HttpsClient::new().map_err(|_| WifiError::HttpError)
    }
}

impl From<PeripheralError> for WifiError {
    fn from(value: PeripheralError) -> Self {
        Self::PeripheralError(value)
    }
}
