use basis::if_mng::*;
use libc;
use std::ffi::CStr;
use std::io::Error;
use std::net::Ipv4Addr;
use ipnet::Ipv4Net;

#[cfg(test)]
mod tests;

extern "C" {
    fn inet_addr(cp: *const libc::c_char) -> libc::in_addr_t;
    fn inet_ntoa(addr: libc::in_addr) -> *const libc::c_char;
}

pub struct InterfaceManagerLibC;

impl InterfaceManager for InterfaceManagerLibC {
    fn get_ip_and_netmask(&self, device_name: &str) -> Result<String, InterfaceError> {
        let ip_res = unsafe {
            DeviceCommand::GetIp(device_name).exec().map_err(|e| InterfaceError(e))?
        };

        let DeviceCommandResult::Addr(ip_addr_str) = ip_res else {
            return Err(InterfaceError("unexpected command result for getting ip".to_owned()))
        };

        let netmask_res = unsafe {
            DeviceCommand::GetNetmask(device_name).exec().map_err(|e| InterfaceError(e))?
        };

        let DeviceCommandResult::Addr(netmask_str) = netmask_res else {
            return Err(InterfaceError("unexpected command result for getting netmask".to_owned()))
        };

        let ip_addr = ip_addr_str.parse::<Ipv4Addr>().map_err(|e| InterfaceError(e.to_string()))?;
        let netmask = netmask_str.parse::<Ipv4Addr>().map_err(|e| InterfaceError(e.to_string()))?;
        let net = Ipv4Net::with_netmask(ip_addr, netmask).map_err(|e| InterfaceError(e.to_string()))?;

        Ok(net.to_string())
    }

    fn up_device(&self, device_name: &str) -> Result<(), InterfaceError> {
        unsafe {
            DeviceCommand::Up(device_name)
                .exec()
                .map_err(|e| InterfaceError(e))
                .map(|_| ())
        }
    }

    fn set_ip_and_netmask(
        &self,
        device_name: &str,
        ip_addr: &Ipv4Addr,
        netmask: &Ipv4Addr,
    ) -> Result<(), InterfaceError> {
        unsafe {
            DeviceCommand::SetIp(device_name, ip_addr)
                .exec()
                .map_err(|e| InterfaceError(e))?;

            DeviceCommand::SetNetmask(device_name, netmask)
                .exec()
                .map_err(|e| InterfaceError(e))
                .map(|_| ())
        }
    }
}

enum DeviceCommandResult {
    Ok,
    Addr(String)
}

enum DeviceCommand<'a> {
    GetIp(&'a str),
    GetNetmask(&'a str),

    SetIp(&'a str, &'a Ipv4Addr),
    SetNetmask(&'a str, &'a Ipv4Addr),
    Up(&'a str),
}

impl<'a> DeviceCommand<'a> {
    unsafe fn open_control_socket() -> Result<i32, String> {
        let s = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
        if s < 0 {
            let msg = format!("control socket problem: {}", Error::last_os_error());
            return Err(msg);
        }
        Ok(s)
    }

    unsafe fn close_control_socket(fd: i32) {
        libc::close(fd);
    }

    unsafe fn get_ip(fd: i32, dev_name: &str) -> Result<DeviceCommandResult, String> {
        let mut ifreq = new_ifreq_for_dev(dev_name);
        if libc::ioctl(fd, libc::SIOCGIFADDR, &mut ifreq) < 0 {
            let msg = format!("can't get ip address: {}", Error::last_os_error());
            return Err(msg);
        }

        let sockaddr = ifreq.ifr_ifru.ifru_addr;
        let addr_in = *(&sockaddr as *const _ as *const libc::sockaddr_in);
        let addr_p = inet_ntoa(addr_in.sin_addr);
        let addr_str = CStr::from_ptr(addr_p).to_str().map_err(|e| e.to_string()).unwrap().to_owned();
        Ok(DeviceCommandResult::Addr(addr_str))
    }

    unsafe fn get_netmask(fd: i32, dev_name: &str) -> Result<DeviceCommandResult, String> {
        let mut ifreq = new_ifreq_for_dev(dev_name);
        if libc::ioctl(fd, libc::SIOCGIFNETMASK, &mut ifreq) < 0 {
            let msg = format!("can't get ip address: {}", Error::last_os_error());
            return Err(msg);
        }

        let sockaddr = ifreq.ifr_ifru.ifru_addr;
        let addr_in = *(&sockaddr as *const _ as *const libc::sockaddr_in);
        let addr_p = inet_ntoa(addr_in.sin_addr);
        let addr_str = CStr::from_ptr(addr_p).to_str().map_err(|e| e.to_string()).unwrap().to_owned();
        Ok(DeviceCommandResult::Addr(addr_str))
    }

    unsafe fn set_ip(fd: i32, dev_name: &str, ip_addr: &str) -> Result<DeviceCommandResult, String> {
        let ifreq = &mut new_ifreq_for_dev(dev_name);

        let ip_sin = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: inet_addr_for_string(ip_addr.to_owned()),
            },
            sin_zero: [0u8; 8],
        };
        (*ifreq).ifr_ifru.ifru_addr = *(&ip_sin as *const _ as *const libc::sockaddr);

        if libc::ioctl(fd, libc::SIOCSIFADDR, ifreq) < 0 {
            let msg = format!("can't set ip address: {}", Error::last_os_error());
            return Err(msg);
        }

        Ok(DeviceCommandResult::Ok)
    }

    unsafe fn set_netmask(fd: i32, dev_name: &str, netmask: &str) -> Result<DeviceCommandResult, String> {
        let ifreq = &mut new_ifreq_for_dev(dev_name);

        let netmask_sin = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: inet_addr_for_string(netmask.to_owned()),
            },
            sin_zero: [0u8; 8],
        };
        (*ifreq).ifr_ifru.ifru_netmask = *(&netmask_sin as *const _ as *const libc::sockaddr);

        if libc::ioctl(fd, libc::SIOCSIFNETMASK, ifreq) < 0 {
            let msg = format!("can't set netmask: {}", Error::last_os_error());
            return Err(msg);
        }

        Ok(DeviceCommandResult::Ok)
    }

    unsafe fn up_device(fd: i32, dev_name: &str) -> Result<DeviceCommandResult, String> {
        let mut ifreq_get = new_ifreq_for_dev(dev_name);

        if libc::ioctl(fd, libc::SIOCGIFFLAGS, &mut ifreq_get) < 0 {
            let msg = format!("can't get device flags: {}", Error::last_os_error());
            return Err(msg);
        }

        let ifreq_set = &mut new_ifreq_for_dev(dev_name);
        (*ifreq_set).ifr_ifru.ifru_flags = ifreq_get.ifr_ifru.ifru_flags | libc::IFF_UP as i16;

        if libc::ioctl(fd, libc::SIOCSIFFLAGS, ifreq_set) < 0 {
            let msg = format!("can't make device up: {}", Error::last_os_error());
            return Err(msg);
        }

        Ok(DeviceCommandResult::Ok)
    }

    unsafe fn exec(&self) -> Result<DeviceCommandResult, String> {
        let fd = Self::open_control_socket()?;

        let res = match self {
            Self::GetIp(dev_name) => Self::get_ip(fd, dev_name),
            Self::GetNetmask(dev_name) => Self::get_netmask(fd, dev_name),
            Self::Up(dev_name) => Self::up_device(fd, dev_name),
            Self::SetIp(dev_name, ip_addr) => {
                Self::set_ip(fd, dev_name, ip_addr.to_string().as_str())
            }
            Self::SetNetmask(dev_name, netmask) => {
                Self::set_netmask(fd, dev_name, netmask.to_string().as_str())
            }
        };

        Self::close_control_socket(fd);
        res
    }
}

// TODO: portability problem -> arm cpu expects str params as u8 instead of i8.
fn new_ifreq_for_dev(dev_name: &str) -> libc::ifreq {
    let mut dev_name_arr = [0u8; 16];
    dev_name_arr[..dev_name.len()].copy_from_slice(dev_name.as_bytes());
    let dev_name_arr_i8 = dev_name_arr.map(|i| i as i8);
    let ifreq = libc::ifreq {
        ifr_name: dev_name_arr_i8,
        ifr_ifru: libc::__c_anonymous_ifr_ifru { ifru_flags: 0 },
    };
    ifreq
}

// TODO: portability problem -> arm cpu expects str params as u8 instead of i8.
unsafe fn inet_addr_for_string(addr: String) -> libc::in_addr_t {
    inet_addr(addr.as_ptr() as *const i8)
}
