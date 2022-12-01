use core::if_mng::InterfaceManager;
use if_mng_libc::InterfaceManagerLibC;
use std::net::Ipv4Addr;

fn main() {
    let dev = InterfaceManagerLibC::new("cli0");

    let ip = "10.0.0.1".parse::<Ipv4Addr>().unwrap();
    let netmask = "255.255.255.0".parse::<Ipv4Addr>().unwrap();
    let res_ip = dev.set_ip_and_netmask(&ip, &netmask);
    println!("set_ip: {:?}", res_ip);
    let dev_up = dev.up_device();
    println!("device_up: {:?}", dev_up);
}
