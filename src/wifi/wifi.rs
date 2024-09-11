use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::modem::{self}, nvs::EspDefaultNvsPartition, timer::EspTaskTimerService, wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi}};

#[derive(Debug)]
pub enum WifiError {
    ConfigurationError,
    StartingError,
    ConnectingError
}

pub struct WifiDriver<'a> {
    controller: AsyncWifi<EspWifi<'a>>,
}

impl <'a>WifiDriver<'a> {
    ///TODO: Docu with Default value of nvs!
    pub fn new(event_loop: EspSystemEventLoop) -> Self {
        let modem = unsafe { modem::Modem::new() };
        let nvs = EspDefaultNvsPartition::take().unwrap();
        let timer_service = EspTaskTimerService::new().unwrap();
        WifiDriver {
            controller: AsyncWifi::wrap(
                EspWifi::new(modem, event_loop.clone(), Some(nvs)).unwrap(),
                event_loop.clone(),
                timer_service,
            ).unwrap()
        }
    }

    pub async fn connect(&mut self, ssid: &str, password: Option<String>) -> Result<(), WifiError> {
        let mut wifi_pass = "".to_string();

        let auth_method = match password {
            Some(pass) => {
                wifi_pass = pass;
                AuthMethod::WPAWPA2Personal
            },
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
        
        self.controller.set_configuration(&wifi_configuration).map_err(|_| WifiError::ConfigurationError)?;

        self.controller.start().await.map_err(|_| WifiError::StartingError)?;
        //TODO: Delete this!
        println!("DEBUG:Wifi started");
    
        self.controller.connect().await.map_err(|_| WifiError::ConnectingError)?;
        //TODO: Delete this!
        println!("DEBUG:wifi connected");
    
        self.controller.wait_netif_up().await.map_err(|_| WifiError::ConnectingError)?;
        //TODO: Delete this!
        println!("DEBUG:wifi netif up"); 

        Ok(())
    }
}
