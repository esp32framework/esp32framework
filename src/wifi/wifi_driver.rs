use crate::microcontroller_src::peripherals::PeripheralError;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::{self},
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use std::net::Ipv4Addr;

use super::http::{Http, HttpClient, HttpsClient};

/// Error types related to WIFI operations.
#[derive(Debug)]
pub enum WifiError {
    ConfigurationError,
    ConnectingError,
    DnsNotFound,
    HttpError,
    InformationError,
    NvsAlreadyTaken,
    PeripheralError(PeripheralError),
    StartingError,
    WifiNotInitialized,
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
    pub fn new(event_loop: EspSystemEventLoop, modem: modem::Modem) -> Result<Self, WifiError> {
        let nvs = EspDefaultNvsPartition::take().map_err(|_| WifiError::NvsAlreadyTaken)?;
        let timer_service = EspTaskTimerService::new().map_err(|_| WifiError::StartingError)?;
        Ok(WifiDriver {
            controller: AsyncWifi::wrap(
                EspWifi::new(modem, event_loop.clone(), Some(nvs))
                    .map_err(|_| WifiError::StartingError)?,
                event_loop.clone(),
                timer_service,
            )
            .map_err(|_| WifiError::StartingError)?,
        })
    }

    /// Connects to the desired wifi network.
    ///
    /// If a password is passed, it connects using the WPAWPA2Personal Authentication method.
    /// Otherwise, it doesn't use an Authentication method.
    ///
    /// # Arguments
    ///
    /// - `ssid`: A &str representing the SSID to connect to.
    /// - `password`: An Option that may contain the password of the SSID.
    ///
    /// # Returns
    ///
    /// A `Result` with Ok if the operation completed successfully, or an `WifiError` if it fails.
    ///
    /// # Errors
    ///
    /// - `WifiError::ConfigurationError`: If the configuration of the wifi driver fails.
    /// - `WifiError::StartingError`: Error while starting wifi driver.
    /// - `WifiError::ConnectingError`: Error while connecting to wifi.
    pub async fn connect(&mut self, ssid: &str, password: Option<String>) -> Result<(), WifiError> {
        let mut wifi_pass = "".to_string();

        let auth_method = match password {
            Some(pass) => {
                wifi_pass = pass;
                AuthMethod::WPAWPA2Personal
            }
            None => AuthMethod::None,
        };

        let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
            ssid: ssid.try_into().unwrap(),
            bssid: None, // MAC address
            auth_method,
            password: (wifi_pass.as_str()).try_into().unwrap(),
            channel: None,
            ..Default::default()
        });

        self.controller
            .set_configuration(&wifi_configuration)
            .map_err(|_| WifiError::ConfigurationError)?;

        self.controller
            .start()
            .await
            .map_err(|_| WifiError::StartingError)?;

        self.controller
            .connect()
            .await
            .map_err(|_| WifiError::ConnectingError)?;

        self.controller
            .wait_netif_up()
            .await
            .map_err(|_| WifiError::ConnectingError)?;

        Ok(())
    }

    /// Checks if the driver is already started.
    ///
    /// # Returns
    ///
    /// A Result containing a bool that indicated whether the driver has started or not. Else,
    /// a `WifiError`.
    ///
    /// # Errors
    ///
    /// - `WifiError::WifiNotInitialized`: If WiFi is not initialized by esp_wifi_init.
    pub fn is_started(&self) -> Result<bool, WifiError> {
        self.controller
            .is_started()
            .map_err(|_| WifiError::WifiNotInitialized)
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
