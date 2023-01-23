use core::wg_mng::*;
use std::ffi::{CStr, CString};
use std::ptr;

mod bindings;

pub struct WireguardManagerCBind;

impl WireguardManager for WireguardManagerCBind {
    fn add_new_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WireguardError> {
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

        let mut priv_key: [u8; 32] = [0; 32];
        let mut pub_key: [u8; 32] = [0; 32];

        let mut priv_key_rep: [i8; 64] = [0; 64];
        let mut pub_key_rep: [i8; 64] = [0; 64];
        let (priv_str, pub_str) = unsafe {
            bindings::wg_generate_private_key(priv_key.as_mut_ptr());
            bindings::wg_generate_public_key(pub_key.as_mut_ptr(), priv_key.as_mut_ptr());

            bindings::wg_key_to_base64(priv_key_rep.as_mut_ptr(), priv_key.as_mut_ptr());
            bindings::wg_key_to_base64(pub_key_rep.as_mut_ptr(), pub_key.as_mut_ptr());

            let priv_str = CStr::from_ptr(priv_key_rep.as_ptr())
                .to_str()
                .map_err(|e| WireguardError(e.to_string()))?;

            let pub_str = CStr::from_ptr(pub_key_rep.as_ptr())
                .to_str()
                .map_err(|e| WireguardError(e.to_string()))?;

            (priv_str, pub_str)
        };

        let set = unsafe {
            (*dev).flags = bindings::wg_device_flags_WGDEVICE_HAS_PRIVATE_KEY
                | bindings::wg_device_flags_WGDEVICE_HAS_PRIVATE_KEY
                | bindings::wg_device_flags_WGDEVICE_HAS_LISTEN_PORT;
            (*dev).private_key = priv_key;
            (*dev).public_key = pub_key;
            (*dev).listen_port = port;

            bindings::wg_set_device(dev)
        };
        if set != 0 {
            return Err(WireguardError("can't set device".to_owned()));
        }

        let wg = WGDevice {
            name: device_name.to_owned(),
            public_key: pub_str.to_owned(),
            private_key: priv_str.to_owned(),
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

            // priv key
            let mut priv_key_rep: [i8; 64] = [0; 64];
            bindings::wg_key_to_base64(priv_key_rep.as_mut_ptr(), (*dev).private_key.as_mut_ptr());
            let priv_str = CStr::from_ptr(priv_key_rep.as_ptr())
                .to_str()
                .map_err(|e| WireguardError(e.to_string()))?;
            wg.private_key = priv_str.to_owned();

            // pub key
            let mut pub_key_rep: [i8; 64] = [0; 64];
            bindings::wg_key_to_base64(pub_key_rep.as_mut_ptr(), (*dev).public_key.as_mut_ptr());
            let pub_str = CStr::from_ptr(pub_key_rep.as_ptr())
                .to_str()
                .map_err(|e| WireguardError(e.to_string()))?;
            wg.public_key = pub_str.to_owned();
        }

        Ok(wg)
    }
}
