use domain::models::wg::{WGDevice, WGPeer};
use std::os::raw::{c_char, c_int, c_longlong, c_ulonglong, c_ushort};

pub const IF_NAMESIZE: usize = 16;
pub const LIBWGSHIM_B64_KEY_SIZE: usize = 45;
pub const ALLOWED_IP_STRLEN: usize = 51;
pub const ENDPOINT_STRLEN: usize = 55;

#[repr(C)]
#[derive(Debug)]
pub enum LibWGShimError {
    NoMem = 1,
    DevNotFound,
    DevAddFailed,
    DevSetFailed,
    PeerNotFound,
}

#[repr(C)]
#[derive(Debug)]
pub struct LibWGShimDevice {
    pub name: [c_char; IF_NAMESIZE],

    pub port: c_ushort,
    pub peers: c_ulonglong,

    pub public_key: [c_char; LIBWGSHIM_B64_KEY_SIZE],
    pub private_key: [c_char; LIBWGSHIM_B64_KEY_SIZE],
}

#[repr(C)]
#[derive(Debug)]
pub struct LibWGShimAllowedIp {
    pub ip_addr: [c_char; ALLOWED_IP_STRLEN],
    pub next: *mut LibWGShimAllowedIp,
}

#[repr(C)]
#[derive(Debug)]
pub struct LibWGShimPeer {
    pub allowed_ip: *mut LibWGShimAllowedIp,

    pub endpoint: [c_char; ENDPOINT_STRLEN],

    pub last_handshake_time: c_longlong,
    pub persistent_keepalive_interval: c_ushort,

    pub rx: c_ulonglong,
    pub tx: c_ulonglong,

    pub public_key: [c_char; LIBWGSHIM_B64_KEY_SIZE],
    pub private_key: [c_char; LIBWGSHIM_B64_KEY_SIZE],
    pub preshared_key: [c_char; LIBWGSHIM_B64_KEY_SIZE],

    pub next: *mut LibWGShimPeer,
}

unsafe extern "C" {
    pub unsafe fn libwgshim_get_device(
        device_name: *const c_char,
        dev: *mut *mut LibWGShimDevice,
    ) -> c_int;

    pub unsafe fn libwgshim_list_device_names() -> *mut c_char;

    pub unsafe fn libwgshim_create_device(
        device_name: *const c_char,
        port: c_ushort,
        dev: *mut *mut LibWGShimDevice,
    ) -> c_int;

    pub unsafe fn libwgshim_delete_device(device_name: *const c_char) -> c_int;

    pub unsafe fn libwgshim_add_peer(
        device_name: *const c_char,
        allowed_ip_head: *mut LibWGShimAllowedIp,
        persistent_keepalive_interval: c_ushort,
        peer: *mut *mut LibWGShimPeer,
    ) -> c_int;

    pub unsafe fn libwgshim_list_peers(
        device_name: *const c_char,
        peer_head: *mut *mut LibWGShimPeer,
    ) -> c_int;

    pub unsafe fn libwgshim_delete_peer(
        device_name: *const c_char,
        public_key: *const c_char,
    ) -> c_int;

    pub unsafe fn libwgshim_free_device(dev: *mut LibWGShimDevice);

    pub unsafe fn libwgshim_free_peer(peer: *mut LibWGShimPeer);
}

fn c_char_array_to_string(buf: &[c_char]) -> String {
    let ptr = buf.as_ptr();
    unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() }
}

impl LibWGShimDevice {
    pub fn to_wg_device(&self) -> WGDevice {
        WGDevice {
            name: c_char_array_to_string(&self.name),
            public_key: c_char_array_to_string(&self.public_key),
            private_key: c_char_array_to_string(&self.private_key),
            port: self.port,
            peers: self.peers,
        }
    }
}

impl LibWGShimAllowedIp {
    pub fn new(ip_addr: &str) -> LibWGShimAllowedIp {
        let mut buf = [0 as c_char; ALLOWED_IP_STRLEN];
        let bytes = ip_addr.as_bytes();

        for (i, &b) in bytes.iter().enumerate().take(ALLOWED_IP_STRLEN - 1) {
            buf[i] = b as c_char;
        }

        LibWGShimAllowedIp {
            ip_addr: buf,
            next: std::ptr::null_mut(),
        }
    }
}

impl LibWGShimPeer {
    pub fn to_wg_peer(&self) -> WGPeer {
        let mut ips = Vec::<String>::new();
        let mut head = self.allowed_ip;

        unsafe {
            while !head.is_null() {
                let ip = c_char_array_to_string(&(*head).ip_addr);
                ips.push(ip);
                head = (*head).next;
            }
        }

        WGPeer {
            allowed_ips: ips,
            endpoint: c_char_array_to_string(&self.endpoint),
            last_handshake_time: self.last_handshake_time,
            persistent_keepalive_interval: self.persistent_keepalive_interval,
            rx: self.rx,
            tx: self.tx,
            public_key: c_char_array_to_string(&self.public_key),
            private_key: c_char_array_to_string(&self.private_key),
            preshared_key: c_char_array_to_string(&self.preshared_key),
        }
    }
}
