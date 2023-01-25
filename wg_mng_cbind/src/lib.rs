use core::wg_mng::*;
use libc;
use std::ffi::{CStr, CString};
use std::ptr;

mod bindings;

extern "C" {
    fn inet_ntoa(addr: libc::in_addr) -> *const libc::c_char;
    fn inet_addr(cp: *const libc::c_char) -> libc::in_addr_t;
}

pub struct WireguardManagerCBind;

impl WireguardManager for WireguardManagerCBind {
    fn add_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WireguardError> {
        let dev_name = CString::new(device_name).map_err(|e| WireguardError(e.to_string()))?;
        let added = unsafe { bindings::wg_add_device(dev_name.as_ptr()) };
        if added != 0 {
            return Err(WireguardError("device can not be added".to_owned()));
        }

        let mut dev: *mut bindings::wg_device = &mut bindings::wg_device {
            name: [0; 16],
            ifindex: 0,
            flags: 0,
            public_key: [0; 32],
            private_key: [0; 32],
            fwmark: 0,
            listen_port: 0,
            first_peer: ptr::null_mut(),
            last_peer: ptr::null_mut(),
        };

        let got = unsafe { bindings::wg_get_device(&mut dev, dev_name.as_ptr()) };
        if got != 0 {
            return Err(WireguardError("can't get device back".to_owned()));
        }

        let mut key_pair = KeyPair::new();

        let set = unsafe {
            (*dev).flags = bindings::wg_device_flags_WGDEVICE_HAS_PRIVATE_KEY
                | bindings::wg_device_flags_WGDEVICE_HAS_PRIVATE_KEY
                | bindings::wg_device_flags_WGDEVICE_HAS_LISTEN_PORT;
            (*dev).private_key = key_pair.private_key;
            (*dev).public_key = key_pair.public_key;
            (*dev).listen_port = port;

            bindings::wg_set_device(dev)
        };
        if set != 0 {
            return Err(WireguardError("can't set device".to_owned()));
        }

        let key_pair_str = key_pair.to_pair_str().map_err(|e| WireguardError(e))?;
        let wg = WGDevice {
            name: device_name.to_owned(),
            public_key: key_pair_str.public_key,
            private_key: key_pair_str.private_key,
            port: port,
            total_peers: 0,
        };

        Ok(wg)
    }

    fn del_device(&self, device_name: &str) -> Result<(), WireguardError> {
        let dev_name = CString::new(device_name).map_err(|e| WireguardError(e.to_string()))?;
        let deleted = unsafe { bindings::wg_del_device(dev_name.as_ptr()) };
        if deleted != 0 {
            return Err(WireguardError("device can not be deleted".to_owned()));
        }

        Ok(())
    }

    fn get_device(&self, device_name: &str) -> Result<WGDevice, WireguardError> {
        let dev_name = CString::new(device_name).map_err(|e| WireguardError(e.to_string()))?;
        let mut dev: *mut bindings::wg_device = &mut bindings::wg_device {
            name: [0; 16],
            ifindex: 0,
            flags: 0,
            public_key: [0; 32],
            private_key: [0; 32],
            fwmark: 0,
            listen_port: 0,
            first_peer: ptr::null_mut(),
            last_peer: ptr::null_mut(),
        };
        let got = unsafe { bindings::wg_get_device(&mut dev, dev_name.as_ptr()) };
        if got != 0 {
            return Err(WireguardError("can't get device".to_owned()));
        }

        let mut wg = WGDevice {
            name: device_name.to_owned(),
            public_key: "".to_owned(),
            private_key: "".to_owned(),
            port: 0,
            total_peers: 0,
        };

        unsafe {
            // port
            wg.port = (*dev).listen_port;

            // total peers
            let mut total_peers = 0;
            let mut peer = (*dev).first_peer;
            loop {
                if peer.is_null() {
                    break;
                }
                total_peers += 1;

                peer = (*peer).next_peer;
            }
            wg.total_peers = total_peers;

            let mut key_pair = KeyPair::new_from((*dev).private_key, (*dev).public_key);
            let key_pair_str = key_pair.to_pair_str().map_err(|e| WireguardError(e))?;

            wg.private_key = key_pair_str.private_key;
            wg.public_key = key_pair_str.public_key;
        }

        Ok(wg)
    }

    fn list_devices(&self) -> Result<Vec<WGDevice>, WireguardError> {
        let devices = unsafe {
            let mut v: Vec<WGDevice> = vec![];

            let names = bindings::wg_list_device_names();
            let mut offset = 0;
            loop {
                let name = CStr::from_ptr(names.offset(offset))
                    .to_str()
                    .map_err(|e| WireguardError(e.to_string()))?;

                if name == "" {
                    break;
                }

                let dev = self.get_device(name)?;
                v.push(dev);
                offset += (name.len() + 1) as isize;
            }
            v
        };

        Ok(devices)
    }

    fn add_peer(
        &self,
        device_name: &str,
        allowed_ips: Vec<String>,
        keepalive: u16,
    ) -> Result<(WGPeer, String), WireguardError> {
        let dev_name = CString::new(device_name).map_err(|e| WireguardError(e.to_string()))?;
        let mut dev: *mut bindings::wg_device = &mut bindings::wg_device {
            name: [0; 16],
            ifindex: 0,
            flags: 0,
            public_key: [0; 32],
            private_key: [0; 32],
            fwmark: 0,
            listen_port: 0,
            first_peer: ptr::null_mut(),
            last_peer: ptr::null_mut(),
        };
        let got = unsafe { bindings::wg_get_device(&mut dev, dev_name.as_ptr()) };
        if got != 0 {
            return Err(WireguardError("can't get device".to_owned()));
        }

        let mut key_pair = KeyPair::new();
        let key_pair_str = key_pair.to_pair_str().map_err(|e| WireguardError(e))?;

        let mut peer: *mut bindings::wg_peer = &mut bindings::wg_peer {
            flags: bindings::wg_peer_flags_WGPEER_HAS_PUBLIC_KEY
                | bindings::wg_peer_flags_WGPEER_HAS_PERSISTENT_KEEPALIVE_INTERVAL,
            public_key: key_pair.public_key,
            preshared_key: [0u8; 32],
            endpoint: bindings::wg_endpoint {
                addr: bindings::sockaddr {
                    sa_family: 0,
                    sa_data: [0; 14],
                },
            },
            last_handshake_time: bindings::timespec64 {
                tv_nsec: 0,
                tv_sec: 0,
            },
            rx_bytes: 0,
            tx_bytes: 0,
            persistent_keepalive_interval: keepalive,
            first_allowedip: ptr::null_mut(),
            last_allowedip: ptr::null_mut(),
            next_peer: ptr::null_mut(),
        };

        let set = unsafe {
            // set allowed ips

            if (*dev).first_peer.is_null() {
                (*dev).first_peer = peer;
            } else {
                let mut last_peer = (*dev).last_peer;
                (*last_peer).next_peer = peer;
                (*dev).last_peer = peer;
            }

            bindings::wg_set_device(dev)
        };

        if set != 0 {
            return Err(WireguardError("can't update device".to_owned()));
        }

        let wg_peer = WGPeer {
            public_key: key_pair_str.public_key,
            persistent_keepalive_interval: keepalive,
            endpoint: "".to_owned(),
            allowed_ips: allowed_ips,
            last_handshake_time: 0,
            rx: 0,
            tx: 0,
        };
        Ok((wg_peer, key_pair_str.private_key))
    }

    fn del_peer(&self, device_name: &str, public_key: &str) -> Result<(), WireguardError> {
        let dev_name = CString::new(device_name).map_err(|e| WireguardError(e.to_string()))?;
        let mut dev: *mut bindings::wg_device = &mut bindings::wg_device {
            name: [0; 16],
            ifindex: 0,
            flags: 0,
            public_key: [0; 32],
            private_key: [0; 32],
            fwmark: 0,
            listen_port: 0,
            first_peer: ptr::null_mut(),
            last_peer: ptr::null_mut(),
        };
        let got = unsafe { bindings::wg_get_device(&mut dev, dev_name.as_ptr()) };
        if got != 0 {
            return Err(WireguardError("can't get device".to_owned()));
        }

        let removed = unsafe {
            let mut peer = (*dev).first_peer;
            loop {
                if peer.is_null() {
                    break;
                }

                let pub_key = KeyPairStr::to_base64((*peer).public_key.as_mut_ptr())
                    .map_err(|e| WireguardError(e))?;
                if pub_key == public_key.to_owned() {
                    (*peer).flags = (*peer).flags | bindings::wg_peer_flags_WGPEER_REMOVE_ME;
                }

                peer = (*peer).next_peer;
            }

            bindings::wg_set_device(dev)
        };
        if removed != 0 {
            return Err(WireguardError("can't remove peer".to_owned()));
        }

        Ok(())
    }

    fn list_peers(&self, device_name: &str) -> Result<Vec<WGPeer>, WireguardError> {
        let dev_name = CString::new(device_name).map_err(|e| WireguardError(e.to_string()))?;
        let mut dev: *mut bindings::wg_device = &mut bindings::wg_device {
            name: [0; 16],
            ifindex: 0,
            flags: 0,
            public_key: [0; 32],
            private_key: [0; 32],
            fwmark: 0,
            listen_port: 0,
            first_peer: ptr::null_mut(),
            last_peer: ptr::null_mut(),
        };
        let got = unsafe { bindings::wg_get_device(&mut dev, dev_name.as_ptr()) };
        if got != 0 {
            return Err(WireguardError("can't get device".to_owned()));
        }

        let mut peers: Vec<WGPeer> = vec![];

        unsafe {
            let mut current_peer = (*dev).first_peer;
            loop {
                if current_peer.is_null() {
                    break;
                }

                let public_key = KeyPairStr::to_base64((*current_peer).public_key.as_mut_ptr())
                    .map_err(|e| WireguardError(e))?;

                let allowed_ips = helpers::allowed_ips_to_vec((*current_peer).first_allowedip)
                    .map_err(|e| WireguardError(e))?;

                let endpoint = helpers::endpoint_to_str((*current_peer).endpoint)
                    .map_err(|e| WireguardError(e))?;

                let wg_peer = WGPeer {
                    public_key: public_key,
                    allowed_ips: allowed_ips,
                    persistent_keepalive_interval: (*current_peer).persistent_keepalive_interval,
                    endpoint: endpoint,
                    last_handshake_time: (*current_peer).last_handshake_time.tv_sec,
                    rx: (*current_peer).rx_bytes,
                    tx: (*current_peer).tx_bytes,
                };
                peers.push(wg_peer);

                current_peer = (*current_peer).next_peer;
            }
        }

        Ok(peers)
    }
}

#[derive(Debug)]
struct KeyPairStr {
    private_key: String,
    public_key: String,
}

impl KeyPairStr {
    fn to_base64(key: *mut u8) -> Result<String, String> {
        let mut rep: [i8; 64] = [0i8; 64];
        unsafe {
            bindings::wg_key_to_base64(rep.as_mut_ptr(), key);

            let s = CStr::from_ptr(rep.as_ptr())
                .to_str()
                .map_err(|e| e.to_string())?
                .to_owned();
            Ok(s)
        }
    }
}

#[derive(Debug)]
struct KeyPair {
    private_key: [u8; 32],
    public_key: [u8; 32],
}

impl KeyPair {
    fn new() -> Self {
        let mut priv_key: [u8; 32] = [0; 32];
        let mut pub_key: [u8; 32] = [0; 32];
        unsafe {
            bindings::wg_generate_private_key(priv_key.as_mut_ptr());
            bindings::wg_generate_public_key(pub_key.as_mut_ptr(), priv_key.as_mut_ptr());
        };

        Self {
            private_key: priv_key,
            public_key: pub_key,
        }
    }

    fn new_from(priv_key: [u8; 32], pub_key: [u8; 32]) -> Self {
        Self {
            private_key: priv_key,
            public_key: pub_key,
        }
    }

    fn to_pair_str(&mut self) -> Result<KeyPairStr, String> {
        let priv_s = KeyPairStr::to_base64(self.private_key.as_mut_ptr())?;
        let pub_s = KeyPairStr::to_base64(self.public_key.as_mut_ptr())?;

        Ok(KeyPairStr {
            private_key: priv_s,
            public_key: pub_s,
        })
    }
}

mod helpers {
    unsafe fn allowed_ip_to_str(
        allowed_ip: crate::bindings::wg_allowedip,
    ) -> Result<String, String> {
        if allowed_ip.family != libc::AF_INET as u16 {
            return Err("ipv6 is not supported yet".to_owned());
        }

        let in_addr = libc::in_addr {
            s_addr: allowed_ip.__bindgen_anon_1.ip4.s_addr,
        };

        let cidr = allowed_ip.cidr.to_string();
        let addr_p = crate::inet_ntoa(in_addr);
        crate::CStr::from_ptr(addr_p)
            .to_str()
            .map(|e| format!("{}/{}", e, cidr))
            .map_err(|e| e.to_string())
    }

    pub unsafe fn allowed_ips_to_vec(
        first: *mut crate::bindings::wg_allowedip,
    ) -> Result<Vec<String>, String> {
        let mut v: Vec<String> = vec![];

        let mut current = first;
        loop {
            if current.is_null() {
                break;
            }

            let addr = allowed_ip_to_str(*current)?;
            v.push(addr);

            current = (*current).next_allowedip;
        }

        Ok(v)
    }

    pub unsafe fn endpoint_to_str(
        endpoint: crate::bindings::wg_endpoint,
    ) -> Result<String, String> {
        if endpoint.addr.sa_family != libc::AF_INET as u16 {
            return Err("ipv6 is not supported yet".to_owned());
        }

        let in_addr = libc::in_addr {
            s_addr: endpoint.addr4.sin_addr.s_addr,
        };

        let port = endpoint.addr4.sin_port.to_be().to_string();
        let addr_p = crate::inet_ntoa(in_addr);
        crate::CStr::from_ptr(addr_p)
            .to_str()
            .map(|e| format!("{}:{}", e, port))
            .map_err(|e| e.to_string())
    }

    pub unsafe fn inet_addr_for_string(addr: String) -> libc::in_addr_t {
        crate::inet_addr(addr.as_ptr() as *const i8)
    }
}
