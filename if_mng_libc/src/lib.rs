use core::if_mng::*;
use libc;
use std::mem::MaybeUninit;
use std::net::Ipv4Addr;

#[cfg(test)]
mod tests;

extern "C" {
    fn inet_addr(cp: *const libc::c_char) -> libc::in_addr_t;
}

pub struct InterfaceManagerLibC<'a> {
    device_name: &'a str,
}

impl<'a> InterfaceManagerLibC<'a> {
    pub fn new(dev_name: &'a str) -> Self {
        Self {
            device_name: dev_name,
        }
    }
}

impl<'a> InterfaceManager for InterfaceManagerLibC<'a> {
    fn get_device_name(&self) -> &str {
        self.device_name
    }

    fn up_device(&self) -> Result<(), InterfaceError> {
        unsafe {
            DeviceCommand::Up(self.get_device_name())
                .exec()
                .map_err(|e| InterfaceError(e))
        }
    }

    fn set_ip_and_netmask(
        &self,
        ip_addr: &Ipv4Addr,
        netmask: &Ipv4Addr,
    ) -> Result<(), InterfaceError> {
        unsafe {
            DeviceCommand::SetIp(self.get_device_name(), ip_addr)
                .exec()
                .map_err(|e| InterfaceError(e))?;

            DeviceCommand::SetNetmask(self.get_device_name(), netmask)
                .exec()
                .map_err(|e| InterfaceError(e))
        }
    }
}

enum DeviceCommand<'a> {
    SetIp(&'a str, &'a Ipv4Addr),
    SetNetmask(&'a str, &'a Ipv4Addr),
    Up(&'a str),
}

impl<'a> DeviceCommand<'a> {
    unsafe fn open_control_socket() -> Result<i32, String> {
        let s = libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0);
        if s < 0 {
            return Err(String::from("control socket problem"));
        }
        Ok(s)
    }

    unsafe fn close_control_socket(fd: i32) {
        libc::close(fd);
    }

    unsafe fn set_ip(fd: i32, dev_name: &str, ip_addr: &str) -> Result<(), String> {
        let ifreq = MaybeUninit::<libc::ifreq>::uninit().as_mut_ptr();

        let mut dev_name_arr = [0u8; 16];
        dev_name_arr[..dev_name.len()].copy_from_slice(dev_name.as_bytes());
        (*ifreq).ifr_name = dev_name_arr;

        let ip_sin = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: inet_addr(ip_addr.to_owned().as_ptr()),
            },
            sin_zero: [0u8; 8],
        };
        (*ifreq).ifr_ifru.ifru_addr = *(&ip_sin as *const _ as *const libc::sockaddr);

        if libc::ioctl(fd, libc::SIOCSIFADDR, ifreq) < 0 {
            return Err(String::from("can't set ip address for device"));
        }

        Ok(())
    }

    unsafe fn set_netmask(fd: i32, dev_name: &str, netmask: &str) -> Result<(), String> {
        let ifreq = MaybeUninit::<libc::ifreq>::uninit().as_mut_ptr();

        let mut dev_name_arr = [0u8; 16];
        dev_name_arr[..dev_name.len()].copy_from_slice(dev_name.as_bytes());
        (*ifreq).ifr_name = dev_name_arr;

        let netmask_sin = libc::sockaddr_in {
            sin_family: libc::AF_INET as u16,
            sin_port: 0,
            sin_addr: libc::in_addr {
                s_addr: inet_addr(netmask.to_owned().as_ptr()),
            },
            sin_zero: [0u8; 8],
        };
        (*ifreq).ifr_ifru.ifru_netmask = *(&netmask_sin as *const _ as *const libc::sockaddr);

        if libc::ioctl(fd, libc::SIOCSIFNETMASK, ifreq) < 0 {
            return Err(String::from("can't set netmask for device"));
        }

        Ok(())
    }

    unsafe fn up_device(fd: i32, dev_name: &str) -> Result<(), String> {
        let ifreq = MaybeUninit::<libc::ifreq>::uninit().as_mut_ptr();

        let mut dev_name_arr = [0u8; 16];
        dev_name_arr[..dev_name.len()].copy_from_slice(dev_name.as_bytes());
        (*ifreq).ifr_name = dev_name_arr;

        if libc::ioctl(fd, libc::SIOCGIFFLAGS, ifreq) < 0 {
            return Err(String::from("can't get device flags"));
        }

        (*ifreq).ifr_ifru.ifru_flags |= libc::IFF_UP as i16;

        if libc::ioctl(fd, libc::SIOCSIFFLAGS, ifreq) < 0 {
            return Err(String::from("can't set device up"));
        }

        Ok(())
    }

    unsafe fn exec(&self) -> Result<(), String> {
        let fd = Self::open_control_socket()?;

        let res = match self {
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
