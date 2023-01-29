mod test_helpers {
    use std::process::{Command, Stdio};

    pub fn wg_show_cmd() -> String {
        let cmd = Command::new("wg").arg("show").output().unwrap();

        std::str::from_utf8(cmd.stdout.as_slice())
            .unwrap()
            .to_owned()
    }

    pub fn ip_link_del_dev(dev_name: &str) {
        let params = format!("link del dev {dev_name}");
        let cmd = Command::new("ip").args(params.split_whitespace()).output();
        assert!(cmd.is_ok());
    }

    pub fn generate_pubkey(private_key: &str) -> String {
        let echo_cmd = Command::new("echo")
            .arg(format!("{}", private_key))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        let echo_out = echo_cmd.stdout.unwrap();

        let wg_pubkey_cmd = Command::new("wg")
            .arg("pubkey")
            .stdin(Stdio::from(echo_out))
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        std::str::from_utf8(wg_pubkey_cmd.wait_with_output().unwrap().stdout.as_slice())
            .unwrap()
            .to_owned()
            .replace("\n", "")
    }
}

#[cfg(test)]
mod wg_mng_tests {
    use super::{super::*, test_helpers};

    #[test]
    fn key_pair_generates_paired() {
        let mut key_pair = KeyPair::new();
        let key_pair_str = key_pair.to_pair_str();
        assert!(key_pair_str.is_ok());

        let pair = key_pair_str.unwrap();
        assert_ne!(pair.private_key, pair.public_key);

        let pub_key_from_cmd = test_helpers::generate_pubkey(pair.private_key.as_str());
        assert_eq!(pub_key_from_cmd, pair.public_key);
    }

    #[test]
    fn add_device_successful() {
        let mngr = WireguardManagerCBind;
        let res = mngr.add_device("test000", 55055);
        assert!(res.is_ok());

        let dev = res.unwrap();
        let wg_show = test_helpers::wg_show_cmd();
        let searches = (
            wg_show.find(dev.name.as_str()),
            wg_show.find(dev.public_key.as_str()),
            wg_show.find(dev.port.to_string().as_str()),
        );
        assert!(searches.0.is_some());
        assert!(searches.1.is_some());
        assert!(searches.2.is_some());

        test_helpers::ip_link_del_dev("test000");
    }

    #[test]
    fn del_device_fails_on_non_existing_device() {
        let mngr = WireguardManagerCBind;
        let res = mngr.del_device("nonexists");
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().to_string(),
            "WireguardError: device can not be deleted"
        )
    }

    #[test]
    fn del_device_successful() {
        let mngr = WireguardManagerCBind;
        let added = mngr.add_device("test001", 55055);
        assert!(added.is_ok());

        let deleted = mngr.del_device("test001");
        assert!(deleted.is_ok());
    }

    #[test]
    fn get_device_fails_on_non_existing_device() {
        let mngr = WireguardManagerCBind;
        let got = mngr.get_device("nonexists");
        assert!(got.is_err());
        assert_eq!(
            got.err().unwrap().to_string(),
            "WireguardError: can't get device"
        );
    }

    #[test]
    fn get_device_successful() {
        let mngr = WireguardManagerCBind;
        let added = mngr.add_device("test002", 55055);
        assert!(added.is_ok());

        let got = mngr.get_device("test002");
        assert!(got.is_ok());

        let added_dev = added.unwrap();
        let got_dev = got.unwrap();
        assert_eq!(added_dev.name, got_dev.name);
        assert_eq!(added_dev.port, got_dev.port);
        assert_eq!(added_dev.private_key, got_dev.private_key);
        assert_eq!(added_dev.public_key, got_dev.public_key);

        test_helpers::ip_link_del_dev("test002");
    }

    #[test]
    fn list_devices_successful() {
        let mngr = WireguardManagerCBind;
        let first = mngr.add_device("test003", 55055);
        let second = mngr.add_device("test004", 44044);
        assert!(first.is_ok());
        assert!(second.is_ok());

        let list = mngr.list_devices();
        assert!(list.is_ok());

        let first_dev = first.unwrap();
        let second_dev = second.unwrap();
        let list_dev = list.unwrap();

        assert!(list_dev.iter().any(|w| w.name == first_dev.name));
        assert!(list_dev.iter().any(|w| w.port == first_dev.port));
        assert!(list_dev
            .iter()
            .any(|w| w.private_key == first_dev.private_key));
        assert!(list_dev
            .iter()
            .any(|w| w.public_key == first_dev.public_key));
        assert!(list_dev.iter().any(|w| w.name == second_dev.name));
        assert!(list_dev.iter().any(|w| w.port == second_dev.port));
        assert!(list_dev
            .iter()
            .any(|w| w.private_key == second_dev.private_key));
        assert!(list_dev
            .iter()
            .any(|w| w.public_key == second_dev.public_key));

        test_helpers::ip_link_del_dev("test003");
        test_helpers::ip_link_del_dev("test004");
    }

    #[test]
    fn add_peer_fails_on_non_existing_device() {
        let mngr = WireguardManagerCBind;
        let res = mngr.add_peer("nonexists", vec!["10.0.0.2/32".to_owned()], 30);
        assert!(res.is_err());
        assert_eq!(
            res.err().unwrap().to_string(),
            "WireguardError: can't get device"
        );
    }

    #[test]
    fn add_peer_fails_on_invalid_ip_format() {
        let mngr = WireguardManagerCBind;
        let added = mngr.add_device("test005", 55055);
        assert!(added.is_ok());

        let peer_add = mngr.add_peer("test005", vec!["10.0.0.2".to_owned()], 30);
        assert!(peer_add.is_err());
        assert_eq!(
            peer_add.err().unwrap().to_string(),
            "WireguardError: invalid ip format: 10.0.0.2"
        );

        test_helpers::ip_link_del_dev("test005");
    }

    #[test]
    fn add_peer_fails_on_invalid_cidr() {
        let mngr = WireguardManagerCBind;
        let added = mngr.add_device("test006", 55055);
        assert!(added.is_ok());

        let peer_add = mngr.add_peer("test006", vec!["10.0.0.2/33".to_owned()], 30);
        assert!(peer_add.is_err());
        assert_eq!(
            peer_add.err().unwrap().to_string(),
            "WireguardError: can't update device"
        );

        test_helpers::ip_link_del_dev("test006");
    }

    #[test]
    fn add_peer_successful() {
        let mngr = WireguardManagerCBind;
        let added = mngr.add_device("test007", 55055);
        assert!(added.is_ok());

        let peer_added = mngr.add_peer(
            "test007",
            vec!["10.0.0.2/32".to_owned(), "10.0.0.3/32".to_owned()],
            30,
        );
        assert!(peer_added.is_ok());

        let wg_show = test_helpers::wg_show_cmd();
        let peer = peer_added.unwrap().0;
        assert!(wg_show.find(peer.public_key.as_str()).is_some());
        assert!(wg_show.find(peer.allowed_ips.join(", ").as_str()).is_some());
        assert!(wg_show
            .find(peer.persistent_keepalive_interval.to_string().as_str())
            .is_some());

        test_helpers::ip_link_del_dev("test007");
    }

    #[test]
    fn del_peer_fails_on_non_existing_device() {
        let mngr = WireguardManagerCBind;

        let del_peer = mngr.del_peer("nonexisting", "public_key");
        assert!(del_peer.is_err());
        assert_eq!(
            del_peer.err().unwrap().to_string(),
            "WireguardError: can't get device"
        );
    }

    #[test]
    fn del_peer_successful() {
        let mngr = WireguardManagerCBind;
        let add = mngr.add_device("test008", 55055);
        assert!(add.is_ok());

        let add_peer = mngr.add_peer("test008", vec!["10.0.0.2/32".to_owned()], 30);
        assert!(add_peer.is_ok());
        let peer = add_peer.unwrap().0;

        let wg_show_before = test_helpers::wg_show_cmd();
        assert!(wg_show_before.find(peer.public_key.as_str()).is_some());

        let del_peer = mngr.del_peer("test008", peer.public_key.as_str());
        assert!(del_peer.is_ok());

        let wg_show_after = test_helpers::wg_show_cmd();
        assert!(wg_show_after.find(peer.public_key.as_str()).is_none());

        test_helpers::ip_link_del_dev("test008");
    }

    #[test]
    fn list_peer_fails_on_non_existing_device() {
        let mngr = WireguardManagerCBind;

        let list_peers = mngr.list_peers("nonexisting");
        assert!(list_peers.is_err());
    }

    #[test]
    fn list_peer_successful() {
        let mngr = WireguardManagerCBind;
        let add = mngr.add_device("test009", 55055);
        assert!(add.is_ok());

        let first_peer_add = mngr.add_peer("test009", vec!["10.0.0.2/32".to_owned()], 30);
        assert!(first_peer_add.is_ok());

        let second_peer_add = mngr.add_peer("test009", vec!["10.0.1.2/32".to_owned()], 45);
        assert!(second_peer_add.is_ok());

        let first_peer = first_peer_add.unwrap().0;
        let second_peer = second_peer_add.unwrap().0;
        let wg_show = test_helpers::wg_show_cmd();
        assert!(wg_show.find(first_peer.public_key.as_str()).is_some());
        assert!(wg_show
            .find(first_peer.allowed_ips.join(", ").as_str())
            .is_some());
        assert!(wg_show
            .find(
                first_peer
                    .persistent_keepalive_interval
                    .to_string()
                    .as_str()
            )
            .is_some());
        assert!(wg_show.find(second_peer.public_key.as_str()).is_some());
        assert!(wg_show
            .find(second_peer.allowed_ips.join(", ").as_str())
            .is_some());
        assert!(wg_show
            .find(
                second_peer
                    .persistent_keepalive_interval
                    .to_string()
                    .as_str()
            )
            .is_some());

        test_helpers::ip_link_del_dev("test009");
    }
}
