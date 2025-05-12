use domain::models::netdev::NetDevIp;
use std::ffi::{CStr, CString};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::os::raw::{c_char, c_int};

pub const IP_NETMASK_STRLEN: usize = 51;

#[repr(C)]
#[derive(Debug)]
pub enum LibNetDevError {
    NoMem = 1,
    CtlSocketFailed,
    NetlinkSocketFailed,
    GetDevFlagsFailed,
    SetDevFlagsFailed,
    InvalidIpStr,
    InvalidIp,
    InvalidIpPrefix,
    DevIpSetFailed,
    DevNetmaskSetFailed,
    DevNotFound,
    NetlinkSendFailed,
    GetifaddrsFailed,
}

#[repr(C)]
#[derive(Debug)]
pub struct LibNetDevIp {
    pub ipv4: [c_char; IP_NETMASK_STRLEN],
    pub ipv6: [c_char; IP_NETMASK_STRLEN],
}

unsafe extern "C" {
    pub unsafe fn libnetdev_get_ip(device_name: *const c_char, ip: *mut *mut LibNetDevIp) -> c_int;

    pub unsafe fn libnetdev_set_ip(device_name: *const c_char, ip: *const LibNetDevIp) -> c_int;

    pub unsafe fn libnetdev_up(device_name: *const c_char) -> c_int;

    pub unsafe fn libnetdev_free_ip(ip: *mut LibNetDevIp);
}

impl LibNetDevIp {
    pub fn to_netdev_ip(&self) -> NetDevIp {
        let ipv4_str = unsafe { CStr::from_ptr(self.ipv4.as_ptr()) }
            .to_str()
            .ok()
            .filter(|s| !s.is_empty());

        let ipv6_str = unsafe { CStr::from_ptr(self.ipv6.as_ptr()) }
            .to_str()
            .ok()
            .filter(|s| !s.is_empty());

        let ipv4 = ipv4_str.and_then(|s| {
            let (ip_str, cidr) = s.split_once('/')?;
            let ip = Ipv4Addr::from_str(ip_str).ok()?;
            let prefix = cidr.parse::<u8>().ok()?;
            Some((ip, prefix))
        });

        let ipv6 = ipv6_str.and_then(|s| {
            let (ip_str, cidr) = s.split_once('/')?;
            let ip = Ipv6Addr::from_str(ip_str).ok()?;
            let prefix = cidr.parse::<u8>().ok()?;
            Some((ip, prefix))
        });

        NetDevIp { ipv4, ipv6 }
    }

    pub fn from_netdev_ip(ip: &NetDevIp) -> Self {
        let mut ipv4 = [0 as c_char; IP_NETMASK_STRLEN];
        let mut ipv6 = [0 as c_char; IP_NETMASK_STRLEN];

        if let Some((addr, prefix)) = ip.ipv4 {
            let ip_str = format!("{}/{}", addr, prefix);
            if let Ok(cstring) = CString::new(ip_str) {
                let bytes = cstring.as_bytes_with_nul();
                ipv4[..bytes.len().min(IP_NETMASK_STRLEN)].copy_from_slice(
                    &bytes.iter().map(|b| *b as c_char).collect::<Vec<_>>(),
                );
            }
        }

        if let Some((addr, prefix)) = ip.ipv6 {
            let ip_str = format!("{}/{}", addr, prefix);
            if let Ok(cstring) = CString::new(ip_str) {
                let bytes = cstring.as_bytes_with_nul();
                ipv6[..bytes.len().min(IP_NETMASK_STRLEN)].copy_from_slice(
                    &bytes.iter().map(|b| *b as c_char).collect::<Vec<_>>(),
                );
            }
        }

        LibNetDevIp { ipv4, ipv6 }
    }
}
