use std::fmt;
use std::net::Ipv4Addr;

#[derive(Debug)]
pub struct InterfaceError(pub String);

impl fmt::Display for InterfaceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "InterfaceError: {}", self.0)
    }
}

impl std::error::Error for InterfaceError {}

pub trait InterfaceManager {
    fn set_ip_and_netmask(
        &self,
        device_name: &str,
        ip_addr: &Ipv4Addr,
        netmask: &Ipv4Addr,
    ) -> Result<(), InterfaceError>;

    fn up_device(&self, device_name: &str) -> Result<(), InterfaceError>;
}
