use crate::WGShimAdapter;
use domain::adapters::wg::WireguardAdapter;

use std::process::Command;

// sudo -E /home/user/.cargo/bin/cargo test -p wgshim -- --test-threads 1
use serial_test::serial;

fn create_wg_device(name: &str) {
    let add_link = Command::new("ip")
        .arg("link")
        .arg("add")
        .arg(name)
        .arg("type")
        .arg("wireguard")
        .output()
        .expect("failed to create wg device");
    println!("add_link = {:?}", add_link);

    Command::new("ip")
        .arg("addr")
        .arg("add")
        .arg("10.0.0.1/24")
        .arg("dev")
        .arg(name)
        .output()
        .expect("failed to set ip address");

    Command::new("wg")
        .arg("set")
        .arg(name)
        .arg("listen-port")
        .arg("51820")
        .output()
        .expect("failed to set listen port");

    let set_peer = Command::new("wg")
        .arg("set")
        .arg(name)
        .arg("peer")
        .arg("CCc0ghN+bKWt176pH6eTWVivrgrSfA1YjPFSa5b9Xho=")
        .arg("allowed-ips")
        .arg("10.0.0.2/32")
        .output()
        .expect("failed to set peer");
    println!("set_peer = {:?}", set_peer);
}

fn delete_wg_device(name: &str) {
    let del_link = Command::new("ip")
        .arg("link")
        .arg("delete")
        .arg(name)
        .output()
        .expect("failed to delete wg device");
    println!("del_link = {:?}", del_link);
}

#[test]
#[serial]
fn test_get_device_non_existing_dev_returns_err() {
    let adapter = WGShimAdapter;
    let result = adapter.get_device("nadev");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "device not found");
    }
}

#[test]
#[serial]
fn test_get_device_with_successful_result() {
    create_wg_device("wgtest1");
    let adapter = WGShimAdapter;
    let result = adapter.get_device("wgtest1");
    assert!(result.is_ok());
    if let Ok(device) = result {
        assert_eq!(device.name, "wgtest1");
        assert_ne!(device.public_key, "");
        assert_ne!(device.private_key, "");
        assert_eq!(device.port, 51820);
        assert_eq!(device.peers, 1);
    }
    delete_wg_device("wgtest1");
}

#[test]
#[serial]
fn test_list_devices() {
    create_wg_device("wgtest2");
    let adapter = WGShimAdapter;
    let result = adapter.list_devices();
    assert!(result.is_ok());
    if let Ok(devices) = result {
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].name, "wgtest2");
        assert_ne!(devices[0].public_key, "");
        assert_ne!(devices[0].private_key, "");
        assert_eq!(devices[0].port, 51820);
        assert_eq!(devices[0].peers, 1);
    }
    delete_wg_device("wgtest2");
}

#[test]
#[serial]
fn test_create_device_fails_if_device_exists() {
    create_wg_device("wgtest3");
    let adapter = WGShimAdapter;
    let result = adapter.create_device("wgtest3", 51820);
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "adding device failed");
    }
    delete_wg_device("wgtest3");
}

#[test]
#[serial]
fn test_create_device_with_successful_result() {
    let adapter = WGShimAdapter;
    let result = adapter.create_device("wgtest4", 51820);
    assert!(result.is_ok());
    if let Ok(device) = result {
        assert_eq!(device.name, "wgtest4");
        assert_ne!(device.public_key, "");
        assert_ne!(device.private_key, "");
        assert_eq!(device.port, 51820);
        assert_eq!(device.peers, 0);
    }
    delete_wg_device("wgtest4");
}

#[test]
#[serial]
fn test_delete_device_non_existing_dev_returns_err() {
    let adapter = WGShimAdapter;
    let result = adapter.delete_device("nadev");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "wireguard error");
    }
}

#[test]
#[serial]
fn test_delete_device_with_successful_result() {
    create_wg_device("wgtest5");
    let adapter = WGShimAdapter;
    let result = adapter.delete_device("wgtest5");
    assert!(result.is_ok());
    if let Ok(_) = result {
        let result = adapter.get_device("wgtest5");
        assert!(result.is_err());
        if let Err(err) = result {
            assert_eq!(err.0, "device not found");
        }
    }
}

#[test]
#[serial]
fn test_list_peers_returns_device_not_found() {
    let adapter = WGShimAdapter;
    let result = adapter.list_peers("nadev");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "device not found");
    }
}

#[test]
#[serial]
fn test_list_peers_returns_a_list_of_peers() {
    create_wg_device("wgtest6");
    let adapter = WGShimAdapter;
    let result = adapter.list_peers("wgtest6");
    assert!(result.is_ok());
    if let Ok(peers) = result {
        assert_eq!(peers.len(), 1);
        assert_eq!(
            peers[0].public_key,
            "CCc0ghN+bKWt176pH6eTWVivrgrSfA1YjPFSa5b9Xho="
        );
        assert_eq!(peers[0].allowed_ips, vec!["10.0.0.2/32".to_owned()]);
        assert_eq!(peers[0].last_handshake_time, 0);
        assert_eq!(peers[0].persistent_keepalive_interval, 0);
        assert_eq!(peers[0].rx, 0);
        assert_eq!(peers[0].tx, 0);
    }
    delete_wg_device("wgtest6");
}

#[test]
#[serial]
fn test_add_peer_returns_device_not_found() {
    let adapter = WGShimAdapter;
    let result = adapter.add_peer("nodev", vec![], 30);
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "device not found");
    }
}

#[test]
#[serial]
fn test_add_peer_returns_successful_result() {
    create_wg_device("wgtest7");
    let adapter = WGShimAdapter;
    let result = adapter.add_peer("wgtest7", vec!["10.0.0.2/32"], 45);
    assert!(result.is_ok());
    if let Ok(peer) = result {
        assert_ne!(peer.public_key, "");
        assert_ne!(peer.private_key, "");
        assert_ne!(peer.preshared_key, "");
        assert_eq!(peer.allowed_ips, vec!["10.0.0.2/32".to_owned()]);
        assert_eq!(peer.persistent_keepalive_interval, 45);
    }
    delete_wg_device("wgtest7");
}

#[test]
#[serial]
fn test_delete_peer_returns_device_not_found() {
    let adapter = WGShimAdapter;
    let result = adapter.delete_peer("nodev", "nodev");
    assert!(result.is_err());
    if let Err(err) = result {
        assert_eq!(err.0, "device not found");
    }
}

#[test]
#[serial]
fn test_delete_peer_returns_successful_result() {
    create_wg_device("wgtest8");
    let adapter = WGShimAdapter;
    let result = adapter.delete_peer("wgtest8", "CCc0ghN+bKWt176pH6eTWVivrgrSfA1YjPFSa5b9Xho=");
    assert!(result.is_ok());
    if let Ok(_) = result {
        let result = adapter.list_peers("wgtest8");
        assert!(result.is_ok());
        if let Ok(peers) = result {
            assert_eq!(peers.len(), 0);
        }
    }
    delete_wg_device("wgtest8");
}
