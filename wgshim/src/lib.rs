use std::ffi::{CStr, CString};
use std::ptr;

use domain::adapters::wg::WireguardAdapter;
use domain::models::wg::*;

mod ffi;

impl TryFrom<i32> for ffi::LibWGShimError {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::NoMem),
            2 => Ok(Self::DevNotFound),
            3 => Ok(Self::DevAddFailed),
            4 => Ok(Self::DevSetFailed),
            5 => Ok(Self::PeerNotFound),
            _ => Err(()),
        }
    }
}

impl From<ffi::LibWGShimError> for WGError {
    fn from(value: ffi::LibWGShimError) -> Self {
        match value {
            ffi::LibWGShimError::NoMem => WGError("no memory".to_owned()),
            ffi::LibWGShimError::DevNotFound => WGError("device not found".to_owned()),
            ffi::LibWGShimError::DevAddFailed => WGError("adding device failed".to_owned()),
            ffi::LibWGShimError::DevSetFailed => WGError("setting device failed".to_owned()),
            ffi::LibWGShimError::PeerNotFound => WGError("peer not found".to_owned()),
        }
    }
}

// Checks the return code from a libwgshim ffi function.
// If the code is non-zero, maps to error to WGError and returns early.
macro_rules! libwgshim_try {
    ($code:expr) => {
        let result: i32 = unsafe { $code };
        if result != 0 {
            let err: WGError = ffi::LibWGShimError::try_from(result)
                .map(|e| e.into())
                .unwrap_or_else(|_| WGError("wireguard error".to_owned()));
            return Err(err);
        }
    };
}

pub struct WGShimAdapter;

impl WireguardAdapter for WGShimAdapter {
    fn get_device(&self, device_name: &str) -> Result<WGDevice, WGError> {
        let dev_name = CString::new(device_name).map_err(|e| WGError(e.to_string()))?;

        let mut dev_ptr: *mut ffi::LibWGShimDevice = ptr::null_mut();
        libwgshim_try!(ffi::libwgshim_get_device(dev_name.as_ptr(), &mut dev_ptr));

        if dev_ptr.is_null() {
            return Err(WGError("wireguard error".to_owned()));
        }

        let shim_dev = unsafe { &(*dev_ptr) };
        let dev = shim_dev.to_wg_device();

        unsafe {
            ffi::libwgshim_free_device(dev_ptr);
        }

        Ok(dev)
    }

    fn list_devices(&self) -> Result<Vec<WGDevice>, WGError> {
        let mut devices: Vec<WGDevice> = vec![];

        let mut offset = 0;
        unsafe {
            let names = ffi::libwgshim_list_device_names();
            loop {
                let dev_name = CStr::from_ptr(names.offset(offset))
                    .to_str()
                    .map_err(|e| WGError(e.to_string()))?;

                if dev_name == "" {
                    break;
                }

                let dev = self.get_device(dev_name).map_err(|e| {
                    libc::free(names as *mut libc::c_void);
                    e
                })?;

                devices.push(dev);
                offset += (dev_name.len() + 1) as isize;
            }

            libc::free(names as *mut libc::c_void);
        }

        Ok(devices)
    }

    fn create_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WGError> {
        let dev_name = CString::new(device_name).map_err(|e| WGError(e.to_string()))?;

        let mut dev_ptr: *mut ffi::LibWGShimDevice = ptr::null_mut();
        libwgshim_try!(ffi::libwgshim_create_device(
            dev_name.as_ptr(),
            port as std::os::raw::c_ushort,
            &mut dev_ptr
        ));

        if dev_ptr.is_null() {
            return Err(WGError("wireguard error".to_owned()));
        }

        let shim_dev = unsafe { &(*dev_ptr) };
        let dev = shim_dev.to_wg_device();

        unsafe {
            ffi::libwgshim_free_device(dev_ptr);
        }

        Ok(dev)
    }

    fn delete_device(&self, device_name: &str) -> Result<(), WGError> {
        let dev_name = CString::new(device_name).map_err(|e| WGError(e.to_string()))?;

        libwgshim_try!(ffi::libwgshim_delete_device(dev_name.as_ptr()));

        Ok(())
    }

    fn list_peers(&self, device_name: &str) -> Result<Vec<WGPeer>, WGError> {
        let dev_name = CString::new(device_name).map_err(|e| WGError(e.to_string()))?;

        let mut head_ptr: *mut ffi::LibWGShimPeer = ptr::null_mut();
        libwgshim_try!(ffi::libwgshim_list_peers(dev_name.as_ptr(), &mut head_ptr));

        let mut peers: Vec<WGPeer> = vec![];

        let mut current = head_ptr;
        unsafe {
            while !current.is_null() {
                let shim_peer = &(*current);
                let wg_peer = shim_peer.to_wg_peer();

                // heap allocation for wg_device causes the list function
                // returning an empty peer, skip it.
                if wg_peer.public_key != "" {
                    peers.push(wg_peer);
                }

                let next = shim_peer.next;
                current = next;
            }

            ffi::libwgshim_free_peer(head_ptr);
        }

        Ok(peers)
    }

    fn add_peer(
        &self,
        device_name: &str,
        allowed_ips: Vec<&str>,
        persistent_keepalive_interval: u16,
    ) -> Result<WGPeer, WGError> {
        let dev_name = CString::new(device_name).map_err(|e| WGError(e.to_string()))?;

        let mut raw_ip_nodes: Vec<*mut ffi::LibWGShimAllowedIp> = allowed_ips
            .iter()
            .map(|s| {
                let b = Box::new(ffi::LibWGShimAllowedIp::new(s));
                Box::into_raw(b)
            })
            .collect();

        for i in 0..raw_ip_nodes.len().saturating_sub(1) {
            unsafe {
                (*raw_ip_nodes[i]).next = raw_ip_nodes[i + 1];
            }
        }

        let allowed_ip_head = raw_ip_nodes.get(0).copied().unwrap_or(ptr::null_mut());

        let mut peer_ptr: *mut ffi::LibWGShimPeer = ptr::null_mut();
        libwgshim_try! {
            ffi::libwgshim_add_peer(dev_name.as_ptr(), allowed_ip_head, persistent_keepalive_interval as std::os::raw::c_ushort, &mut peer_ptr)
        };

        let shim_peer = unsafe { &(*peer_ptr) };
        let peer = shim_peer.to_wg_peer();

        unsafe {
            ffi::libwgshim_free_peer(peer_ptr);
        }

        Ok(peer)
    }

    fn delete_peer(&self, device_name: &str, public_key: &str) -> Result<(), WGError> {
        let dev_name = CString::new(device_name).map_err(|e| WGError(e.to_string()))?;
        let pk = CString::new(public_key).map_err(|e| WGError(e.to_string()))?;

        libwgshim_try!(ffi::libwgshim_delete_peer(dev_name.as_ptr(), pk.as_ptr()));

        Ok(())
    }
}
