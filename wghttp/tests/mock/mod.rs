use domain::adapters::netdev::NetworkDeviceAdapter;
use domain::adapters::wg::WireguardAdapter;
use domain::models::netdev::*;
use domain::models::wg::*;

#[cfg(test)]
pub struct WireguardMockAdapter {
    get_fn: fn(&str) -> Result<WGDevice, WGError>,
    list_fn: fn() -> Result<Vec<WGDevice>, WGError>,
    create_fn: fn(&str, u16) -> Result<WGDevice, WGError>,
    delete_fn: fn(&str) -> Result<(), WGError>,
    list_peers_fn: fn(&str) -> Result<Vec<WGPeer>, WGError>,
    add_peer_fn: fn(&str, Vec<&str>, u16) -> Result<WGPeer, WGError>,
    delete_peer_fn: fn(&str, &str) -> Result<(), WGError>,
}

#[cfg(test)]
impl WireguardAdapter for WireguardMockAdapter {
    fn get_device(&self, device_name: &str) -> Result<WGDevice, WGError> {
        (self.get_fn)(device_name)
    }

    fn list_devices(&self) -> Result<Vec<WGDevice>, WGError> {
        (self.list_fn)()
    }

    fn create_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WGError> {
        (self.create_fn)(device_name, port)
    }

    fn delete_device(&self, device_name: &str) -> Result<(), WGError> {
        (self.delete_fn)(device_name)
    }

    fn list_peers(&self, device_name: &str) -> Result<Vec<WGPeer>, WGError> {
        (self.list_peers_fn)(device_name)
    }

    fn add_peer(
        &self,
        device_name: &str,
        allowed_ips: Vec<&str>,
        persistent_keepalive_interval: u16,
    ) -> Result<WGPeer, WGError> {
        (self.add_peer_fn)(device_name, allowed_ips, persistent_keepalive_interval)
    }

    fn delete_peer(&self, device_name: &str, public_key: &str) -> Result<(), WGError> {
        (self.delete_peer_fn)(device_name, public_key)
    }
}

impl WireguardMockAdapter {
    pub fn new(
        get_fn: Option<fn(&str) -> Result<WGDevice, WGError>>,
        list_fn: Option<fn() -> Result<Vec<WGDevice>, WGError>>,
        create_fn: Option<fn(&str, u16) -> Result<WGDevice, WGError>>,
        delete_fn: Option<fn(&str) -> Result<(), WGError>>,
        list_peers_fn: Option<fn(&str) -> Result<Vec<WGPeer>, WGError>>,
        add_peer_fn: Option<fn(&str, Vec<&str>, u16) -> Result<WGPeer, WGError>>,
        delete_peer_fn: Option<fn(&str, &str) -> Result<(), WGError>>,
    ) -> Self {
        WireguardMockAdapter {
            get_fn: get_fn.unwrap_or(|_| Err(WGError("not found".to_owned()))),
            list_fn: list_fn.unwrap_or(|| Ok(vec![])),
            create_fn: create_fn.unwrap_or(|_, _| Err(WGError("not found".to_owned()))),
            delete_fn: delete_fn.unwrap_or(|_| Err(WGError("not found".to_owned()))),
            list_peers_fn: list_peers_fn.unwrap_or(|_| Ok(vec![])),
            add_peer_fn: add_peer_fn.unwrap_or(|_, _, _| Err(WGError("not found".to_owned()))),
            delete_peer_fn: delete_peer_fn.unwrap_or(|_, _| Err(WGError("not found".to_owned()))),
        }
    }
}

#[cfg(test)]
pub struct NetworkDeviceMockAdapter {
    get_ip_fn: fn(&str) -> Result<NetDevIp, NetDevError>,
    set_ip_fn: fn(&str, &NetDevIp) -> Result<(), NetDevError>,
    up_fn: fn(&str) -> Result<(), NetDevError>,
}

#[cfg(test)]
impl NetworkDeviceAdapter for NetworkDeviceMockAdapter {
    fn get_ip(&self, device_name: &str) -> Result<NetDevIp, NetDevError> {
        (self.get_ip_fn)(device_name)
    }

    fn set_ip(&self, device_name: &str, ip: &NetDevIp) -> Result<(), NetDevError> {
        (self.set_ip_fn)(device_name, ip)
    }

    fn up(&self, device_name: &str) -> Result<(), NetDevError> {
        (self.up_fn)(device_name)
    }
}

impl NetworkDeviceMockAdapter {
    pub fn new(
        get_ip_fn: Option<fn(&str) -> Result<NetDevIp, NetDevError>>,
        set_ip_fn: Option<fn(&str, &NetDevIp) -> Result<(), NetDevError>>,
        up_fn: Option<fn(&str) -> Result<(), NetDevError>>,
    ) -> Self {
        NetworkDeviceMockAdapter {
            get_ip_fn: get_ip_fn.unwrap_or(|_| Err(NetDevError("not found".to_owned()))),
            set_ip_fn: set_ip_fn.unwrap_or(|_, _| Err(NetDevError("not found".to_owned()))),
            up_fn: up_fn.unwrap_or(|_| Err(NetDevError("not found".to_owned()))),
        }
    }
}
