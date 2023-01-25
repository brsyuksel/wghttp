use std::fmt;

#[derive(Debug)]
pub struct WireguardError(pub String);

impl fmt::Display for WireguardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "WireguardError: {}", self.0)
    }
}

impl std::error::Error for WireguardError {}

#[derive(Debug)]
pub struct WGDevice {
    pub name: String,
    pub public_key: String,
    pub private_key: String,
    pub port: u16,
    pub total_peers: u64,
}

#[derive(Debug)]
pub struct WGPeer {
    pub public_key: String,
    pub allowed_ips: Vec<String>,
    pub persistent_keepalive_interval: u16,
    pub endpoint: String,
    pub last_handshake_time: i64,
    pub rx: u64,
    pub tx: u64,
}

pub trait WireguardManager {
    fn add_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WireguardError>;
    fn del_device(&self, device_name: &str) -> Result<(), WireguardError>;
    fn get_device(&self, device_name: &str) -> Result<WGDevice, WireguardError>;
    fn list_devices(&self) -> Result<Vec<WGDevice>, WireguardError>;

    fn add_peer(&self, device_name: &str, allowed_ips: Vec<String>, keepalive: u16) -> Result<(WGPeer, String), WireguardError>;
    fn del_peer(&self, device_name: &str, public_key: &str) -> Result<(), WireguardError>;
    fn list_peers(&self, device_name: &str) -> Result<Vec<WGPeer>, WireguardError>;
}
