use core::wg_mng::*;
use std::ffi::{CStr, CString};
use std::ptr;

mod bindings;

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
}

#[derive(Debug)]
struct KeyPair {
    private_key: [u8; 32],
    public_key: [u8; 32],
}

#[derive(Debug)]
struct KeyPairStr {
    private_key: String,
    public_key: String,
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
        let mut priv_key_rep: [i8; 64] = [0; 64];
        let mut pub_key_rep: [i8; 64] = [0; 64];

        let (priv_str, pub_str) = unsafe {
            bindings::wg_key_to_base64(priv_key_rep.as_mut_ptr(), self.private_key.as_mut_ptr());
            bindings::wg_key_to_base64(pub_key_rep.as_mut_ptr(), self.public_key.as_mut_ptr());

            let priv_s = CStr::from_ptr(priv_key_rep.as_ptr())
                .to_str()
                .map_err(|e| e.to_string())?;

            let pub_s = CStr::from_ptr(pub_key_rep.as_ptr())
                .to_str()
                .map_err(|e| e.to_string())?;

            (priv_s.to_owned(), pub_s.to_owned())
        };

        Ok(KeyPairStr {
            private_key: priv_str,
            public_key: pub_str,
        })
    }
}
