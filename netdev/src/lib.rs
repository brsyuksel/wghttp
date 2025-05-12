use std::ffi::CString;
use std::ptr;

use domain::adapters::netdev::NetworkDeviceAdapter;
use domain::models::netdev::*;

mod ffi;

use std::convert::TryFrom;

impl TryFrom<i32> for ffi::LibNetDevError {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::NoMem),
            2 => Ok(Self::CtlSocketFailed),
            3 => Ok(Self::NetlinkSocketFailed),
            4 => Ok(Self::GetDevFlagsFailed),
            5 => Ok(Self::SetDevFlagsFailed),
            6 => Ok(Self::InvalidIpStr),
            7 => Ok(Self::InvalidIp),
            8 => Ok(Self::InvalidIpPrefix),
            9 => Ok(Self::DevIpSetFailed),
            10 => Ok(Self::DevNetmaskSetFailed),
            11 => Ok(Self::DevNotFound),
            12 => Ok(Self::NetlinkSendFailed),
            13 => Ok(Self::GetifaddrsFailed),
            _ => Err(()),
        }
    }
}

impl From<ffi::LibNetDevError> for NetDevError {
    fn from(err: ffi::LibNetDevError) -> Self {
        let msg = match err {
            ffi::LibNetDevError::NoMem => "Memory allocation failed",
            ffi::LibNetDevError::CtlSocketFailed => "Failed to open control socket",
            ffi::LibNetDevError::NetlinkSocketFailed => "Failed to open netlink socket",
            ffi::LibNetDevError::GetDevFlagsFailed => "Failed to get interface flags",
            ffi::LibNetDevError::SetDevFlagsFailed => "Failed to set interface flags",
            ffi::LibNetDevError::InvalidIpStr => "Invalid IP string format",
            ffi::LibNetDevError::InvalidIp => "Invalid IP address",
            ffi::LibNetDevError::InvalidIpPrefix => "Invalid IP prefix length",
            ffi::LibNetDevError::DevIpSetFailed => "Failed to set device IP",
            ffi::LibNetDevError::DevNetmaskSetFailed => "Failed to set device netmask",
            ffi::LibNetDevError::DevNotFound => "Device not found",
            ffi::LibNetDevError::NetlinkSendFailed => "Failed to send netlink message",
            ffi::LibNetDevError::GetifaddrsFailed => "getifaddrs() system call failed",
        };

        NetDevError(msg.to_string())
    }
}

macro_rules! libnetdev_try {
    ($code:expr) => {
        let result: i32 = unsafe { $code };
        if result != 0 {
            let err: NetDevError = ffi::LibNetDevError::try_from(result)
                .map(|e| e.into())
                .unwrap_or_else(|_| NetDevError("network device error".to_owned()));
            return Err(err);
        }
    };
}

pub struct NetDevAdapter;

impl NetworkDeviceAdapter for NetDevAdapter {
    fn get_ip(&self, device_name: &str) -> Result<NetDevIp, NetDevError> {
        let dev_name = CString::new(device_name).map_err(|e| NetDevError(e.to_string()))?;

        let mut ip_ptr: *mut ffi::LibNetDevIp = ptr::null_mut();
        libnetdev_try!(ffi::libnetdev_get_ip(dev_name.as_ptr(), &mut ip_ptr));

        if ip_ptr.is_null() {
            return Err(NetDevError("network device error".to_owned()));
        }

        let libnetdev_ip = unsafe { &(*ip_ptr) };
        let ip = libnetdev_ip.to_netdev_ip();

        unsafe {
            ffi::libnetdev_free_ip(ip_ptr);
        }

        Ok(ip)
    }

    fn set_ip(&self, device_name: &str, ip: &NetDevIp) -> Result<(), NetDevError> {
        let dev_name = CString::new(device_name).map_err(|e| NetDevError(e.to_string()))?;

        let libnetdev_ip = &ffi::LibNetDevIp::from_netdev_ip(ip);
        libnetdev_try!(ffi::libnetdev_set_ip(dev_name.as_ptr(), libnetdev_ip));

        Ok(())
    }

    fn up(&self, device_name: &str) -> Result<(), NetDevError> {
        let dev_name = CString::new(device_name).map_err(|e| NetDevError(e.to_string()))?;

        libnetdev_try!(ffi::libnetdev_up(dev_name.as_ptr()));

        Ok(())
    }
}
