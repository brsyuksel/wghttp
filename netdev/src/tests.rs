use crate::NetDevAdapter;
use domain::adapters::netdev::NetworkDeviceAdapter;
use domain::models::netdev::*;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::process::Command;

fn create_dummy_device(name: &str) {
    Command::new("ip")
        .arg("link")
        .arg("add")
        .arg(name)
        .arg("type")
        .arg("dummy")
        .output()
        .expect("failed to create dummy device");
}

fn delete_dummy_device(name: &str) {
    Command::new("ip")
        .arg("link")
        .arg("delete")
        .arg(name)
        .output()
        .expect("failed to delete dummy device");
}

fn get_dummy_device_status(name: &str) -> String {
    let output = Command::new("ip")
        .arg("link")
        .arg("show")
        .arg(name)
        .output()
        .expect("failed to get device status");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn set_dummy_device_ip(name: &str, ip: &str) {
    Command::new("ip")
        .arg("addr")
        .arg("add")
        .arg(ip)
        .arg("dev")
        .arg(name)
        .output()
        .expect("failed to set IP address");
}

#[test]
fn test_netdev_get_ip_non_existing_dev_returns_error() {
    let adapter = NetDevAdapter;
    let result = adapter.get_ip("non_existing_device");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "device not found");
    }
}

#[test]
fn test_netdev_get_ip() {
    create_dummy_device("test0");
    set_dummy_device_ip("test0", "10.0.0.1/24");
    set_dummy_device_ip("test0", "2001:db8::1/64");
    let adapter = NetDevAdapter;
    let result = adapter.get_ip("test0");
    assert!(result.is_ok());
    if let Ok(ip) = result {
        assert_eq!(ip.ipv4_str(), Some("10.0.0.1/24".to_string()));
        assert_eq!(ip.ipv6_str(), Some("2001:db8::1/64".to_string()));
    }

    delete_dummy_device("test0");
}

#[test]
fn test_netdev_set_ip_invalid_ipv4_prefix() {
    create_dummy_device("test1");
    let adapter = NetDevAdapter;
    let result = adapter.set_ip(
        "test1",
        &NetDevIp::new(Some((IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 33)), None),
    );
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "invalid ip prefix length");
    }
    delete_dummy_device("test1");
}

#[test]
fn test_netdev_set_ip_invalid_ipv6_prefix() {
    create_dummy_device("test2");
    let adapter = NetDevAdapter;
    let result = adapter.set_ip(
        "test2",
        &NetDevIp::new(
            None,
            Some((
                IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1)),
                129,
            )),
        ),
    );
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "invalid ip prefix length");
    }
    delete_dummy_device("test2");
}

#[test]
fn test_netdev_set_ip_failed() {
    create_dummy_device("testx");
    set_dummy_device_ip("testx", "10.0.0.2/32");

    create_dummy_device("test3");
    let adapter = NetDevAdapter;
    let result = adapter.set_ip(
        "test3",
        &NetDevIp::new(Some((IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 32)), None),
    );

    assert!(result.is_err());
    if let Err(err) = result {
        // ioctl does not fail for same ip until netmask is set
        assert_eq!(err.0, "failed to set device netmask");
    }

    delete_dummy_device("test3");
    delete_dummy_device("testx");
}

#[test]
fn test_netdev_set_ip_successful_result() {
    create_dummy_device("test4");
    let adapter = NetDevAdapter;
    let result = adapter.set_ip(
        "test4",
        &NetDevIp::new(
            Some((IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)), 24)),
            Some((
                IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1)),
                64,
            )),
        ),
    );
    assert!(result.is_ok());
    delete_dummy_device("test4");
}

#[test]
fn test_netdev_up_failed() {
    let adapter = NetDevAdapter;
    let result = adapter.up("testy");
    assert!(result.is_err());
}

#[test]
fn test_netdev_up() {
    create_dummy_device("test5");
    let adapter = NetDevAdapter;
    let result = adapter.up("test5");
    assert!(result.is_ok());
    let status = get_dummy_device_status("test5");
    assert!(status.contains("UP"));
    delete_dummy_device("test5");
}
