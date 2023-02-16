mod test_helpers {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use basis::if_mng::*;
    use basis::wg_mng::*;

    pub fn create_wg_mngr(
        fails: bool,
        empty_list_dev: bool,
        empty_list_peer: bool,
    ) -> impl WireguardManager {
        struct MockWG {
            fails: bool,
            empty_list_dev: bool,
            empty_list_peer: bool,
        }

        impl WireguardManager for MockWG {
            fn add_device(&self, device_name: &str, port: u16) -> Result<WGDevice, WireguardError> {
                if self.fails {
                    return Err(WireguardError("add_device fails".to_owned()));
                }

                Ok(WGDevice {
                    name: device_name.to_owned(),
                    public_key: "pubkey".to_owned(),
                    private_key: "privkey".to_owned(),
                    port: port,
                    total_peers: 0,
                })
            }

            fn del_device(&self, device_name: &str) -> Result<(), WireguardError> {
                if self.fails {
                    return Err(WireguardError("del_device fails".to_owned()));
                }

                Ok(())
            }

            fn get_device(&self, device_name: &str) -> Result<WGDevice, WireguardError> {
                if self.fails {
                    return Err(WireguardError("get_device fails".to_owned()));
                }

                Ok(WGDevice {
                    name: device_name.to_owned(),
                    public_key: "pubkey".to_owned(),
                    private_key: "privkey".to_owned(),
                    port: 11011,
                    total_peers: 11,
                })
            }

            fn list_devices(&self) -> Result<Vec<WGDevice>, WireguardError> {
                if self.fails {
                    return Err(WireguardError("list_devices fails".to_owned()));
                }

                if self.empty_list_dev {
                    return Ok(vec![]);
                }

                let devs = vec![
                    WGDevice {
                        name: "dev0".to_owned(),
                        public_key: "pub0".to_owned(),
                        private_key: "priv0".to_owned(),
                        port: 11011,
                        total_peers: 11,
                    },
                    WGDevice {
                        name: "dev1".to_owned(),
                        public_key: "pub1".to_owned(),
                        private_key: "priv1".to_owned(),
                        port: 22022,
                        total_peers: 22,
                    },
                ];

                Ok(devs)
            }

            fn add_peer(
                &self,
                device_name: &str,
                allowed_ips: Vec<String>,
                keepalive: u16,
            ) -> Result<(WGPeer, String), WireguardError> {
                if self.fails {
                    return Err(WireguardError("add_peer fails".to_owned()));
                }

                Ok((
                    WGPeer {
                        public_key: "pubkey".to_owned(),
                        allowed_ips: allowed_ips,
                        persistent_keepalive_interval: keepalive,
                        endpoint: "".to_owned(),
                        last_handshake_time: 0,
                        rx: 11,
                        tx: 11,
                    },
                    "privkey".to_owned(),
                ))
            }

            fn del_peer(&self, device_name: &str, public_key: &str) -> Result<(), WireguardError> {
                if self.fails {
                    return Err(WireguardError("del_peer fails".to_owned()));
                }

                Ok(())
            }

            fn list_peers(&self, device_name: &str) -> Result<Vec<WGPeer>, WireguardError> {
                if self.fails {
                    return Err(WireguardError("list_peers fails".to_owned()));
                }

                if self.empty_list_peer {
                    return Ok(vec![]);
                }

                let peers = vec![
                    WGPeer {
                        public_key: "pub0".to_owned(),
                        allowed_ips: vec!["10.0.0.1/32".to_owned()],
                        persistent_keepalive_interval: 15,
                        endpoint: "".to_owned(),
                        last_handshake_time: 1,
                        rx: 11,
                        tx: 11,
                    },
                    WGPeer {
                        public_key: "pub1".to_owned(),
                        allowed_ips: vec!["10.0.0.2/32".to_owned()],
                        persistent_keepalive_interval: 30,
                        endpoint: "".to_owned(),
                        last_handshake_time: 2,
                        rx: 22,
                        tx: 22,
                    },
                ];

                Ok(peers)
            }
        }

        unsafe impl Send for MockWG {}

        MockWG {
            fails: fails,
            empty_list_dev: empty_list_dev,
            empty_list_peer: empty_list_peer,
        }
    }

    pub fn create_if_mngr(ip_fails: bool, up_fails: bool) -> impl InterfaceManager {
        struct MockIF {
            ip_fails: bool,
            up_fails: bool,
        }

        impl InterfaceManager for MockIF {
            fn set_ip_and_netmask(
                &self,
                device_name: &str,
                ip_addr: &std::net::Ipv4Addr,
                netmask: &std::net::Ipv4Addr,
            ) -> Result<(), InterfaceError> {
                if self.ip_fails {
                    return Err(InterfaceError("set_ip_and_netmask fails".to_owned()));
                }
                Ok(())
            }

            fn up_device(&self, device_name: &str) -> Result<(), InterfaceError> {
                if self.up_fails {
                    return Err(InterfaceError("up_device fails".to_owned()));
                }
                Ok(())
            }
        }

        unsafe impl Send for MockIF {}

        MockIF {
            ip_fails: ip_fails,
            up_fails: up_fails,
        }
    }

    pub fn sync_mngr<T>(mngr: T) -> Arc<Mutex<T>> {
        Arc::new(Mutex::new(mngr))
    }
}

#[cfg(test)]
mod handlers_tests {
    use warp::http::StatusCode;
    use warp::test::request;

    use super::test_helpers;
    use crate::api::models::*;
    use crate::server;

    #[tokio::test]
    async fn list_devices_returns_error_for_any_wg_error() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);
        let filter = server::list_devices(sync_wg);

        let resp = request()
            .path("/devices")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR)
    }

    #[tokio::test]
    async fn list_devices_returns_empty_list() {
        let wg = test_helpers::create_wg_mngr(false, true, false);
        let sync_wg = test_helpers::sync_mngr(wg);
        let filter = server::list_devices(sync_wg);

        let resp = request()
            .path("/devices")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "[]")
    }

    #[tokio::test]
    async fn list_devices_returns_list_of_devs() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);
        let filter = server::list_devices(sync_wg);

        let resp = request()
            .path("/devices")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "[{\"device_name\":\"dev0\",\"port\":11011,\"total_peers\":11},{\"device_name\":\"dev1\",\"port\":22022,\"total_peers\":22}]")
    }

    #[tokio::test]
    async fn create_device_returns_400_for_invalid_ip_format() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(false, false);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::create_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices")
            .method("POST")
            .json(&CreateDevice {
                device_name: "test0".to_owned(),
                port: 11011,
                ip: "10.0.0.2/322".to_owned(),
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(resp.body(), "{\"message\":\"error on parsing ip addr\"}")
    }

    #[tokio::test]
    async fn create_device_returns_server_err_for_wg_error() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(false, false);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::create_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices")
            .method("POST")
            .json(&CreateDevice {
                device_name: "test0".to_owned(),
                port: 11011,
                ip: "10.0.0.2/32".to_owned(),
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(resp.body(), "{\"message\":\"add_device fails\"}")
    }

    #[tokio::test]
    async fn create_device_returns_conflict_on_err_setting_ip() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(true, false);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::create_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices")
            .method("POST")
            .json(&CreateDevice {
                device_name: "test0".to_owned(),
                port: 11011,
                ip: "10.0.0.2/32".to_owned(),
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::CONFLICT);
        assert_eq!(resp.body(), "{\"message\":\"set_ip_and_netmask fails\"}")
    }

    #[tokio::test]
    async fn create_device_returns_conflict_on_setting_dev_up() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(false, true);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::create_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices")
            .method("POST")
            .json(&CreateDevice {
                device_name: "test0".to_owned(),
                port: 11011,
                ip: "10.0.0.2/32".to_owned(),
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::CONFLICT);
        assert_eq!(resp.body(), "{\"message\":\"up_device fails\"}")
    }

    #[tokio::test]
    async fn create_device_returns_dev_detail_with_201() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(false, false);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::create_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices")
            .method("POST")
            .json(&CreateDevice {
                device_name: "test0".to_owned(),
                port: 11011,
                ip: "10.0.0.2/32".to_owned(),
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::CREATED);
        assert_eq!(resp.body(), "{\"device_name\":\"test0\",\"port\":11011,\"ip\":\"10.0.0.2/32\",\"public_key\":\"pubkey\",\"private_key\":\"privkey\",\"total_peers\":0}")
    }

    #[tokio::test]
    async fn get_device_returns_404_for_non_existing_dev() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(false, false);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::get_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices/test0")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.body(), "{\"message\":\"get_device fails\"}")
    }

    #[tokio::test]
    async fn get_device_returns_dev_details() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let im = test_helpers::create_if_mngr(false, false);
        let sync_im = test_helpers::sync_mngr(im);

        let filter = server::get_device(sync_wg, sync_im);

        let resp = request()
            .path("/devices/test0")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "{\"device_name\":\"test0\",\"port\":11011,\"ip\":\"\",\"public_key\":\"pubkey\",\"private_key\":\"privkey\",\"total_peers\":11}")
    }

    #[tokio::test]
    async fn delete_device_returns_404_for_non_existing_dev() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::delete_device(sync_wg);

        let resp = request()
            .path("/devices/test0")
            .method("DELETE")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND)
    }

    #[tokio::test]
    async fn delete_device_returns_204_for_successful_deletion() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::delete_device(sync_wg);

        let resp = request()
            .path("/devices/test0")
            .method("DELETE")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT)
    }

    #[tokio::test]
    async fn list_peers_returns_404_for_non_existing_dev() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::list_peers(sync_wg);

        let resp = request()
            .path("/devices/test0/peers")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.body(), "{\"message\":\"list_peers fails\"}")
    }

    #[tokio::test]
    async fn list_peers_returns_empty_list() {
        let wg = test_helpers::create_wg_mngr(false, false, true);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::list_peers(sync_wg);

        let resp = request()
            .path("/devices/test0/peers")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "[]")
    }

    #[tokio::test]
    async fn list_peers_returns_list_of_peers() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::list_peers(sync_wg);

        let resp = request()
            .path("/devices/test0/peers")
            .method("GET")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "[{\"public_key\":\"pub0\",\"endpoint\":\"\",\"last_handshake_time\":1,\"rx\":11,\"tx\":11,\"persistent_keepalive_time\":15,\"allowed_ips\":[\"10.0.0.1/32\"]},{\"public_key\":\"pub1\",\"endpoint\":\"\",\"last_handshake_time\":2,\"rx\":22,\"tx\":22,\"persistent_keepalive_time\":30,\"allowed_ips\":[\"10.0.0.2/32\"]}]")
    }

    #[tokio::test]
    async fn create_peer_returns_400_for_invalid_ip_format() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::create_peer(sync_wg);

        let resp = request()
            .path("/devices/test0/peers")
            .method("POST")
            .json(&CreatePeer {
                persistent_keepalive_time: 15,
                allowed_ips: vec!["10.0.0.1/32".to_owned(), "10.0.0.2/322".to_owned()],
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
        assert_eq!(resp.body(), "{\"message\":\"error on parsing ip addr\"}")
    }

    #[tokio::test]
    async fn create_peer_returns_404_for_non_existing_device() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::create_peer(sync_wg);

        let resp = request()
            .path("/devices/test0/peers")
            .method("POST")
            .json(&CreatePeer {
                persistent_keepalive_time: 15,
                allowed_ips: vec!["10.0.0.1/32".to_owned(), "10.0.0.2/32".to_owned()],
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
        assert_eq!(resp.body(), "{\"message\":\"add_peer fails\"}")
    }

    #[tokio::test]
    async fn create_peer_returns_peer_detail_with_priv_key() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::create_peer(sync_wg);

        let resp = request()
            .path("/devices/test0/peers")
            .method("POST")
            .json(&CreatePeer {
                persistent_keepalive_time: 15,
                allowed_ips: vec!["10.0.0.1/32".to_owned(), "10.0.0.2/32".to_owned()],
            })
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(resp.body(), "{\"public_key\":\"pubkey\",\"private_key\":\"privkey\",\"endpoint\":\"\",\"last_handshake_time\":0,\"rx\":11,\"tx\":11,\"persistent_keepalive_time\":15,\"allowed_ips\":[\"10.0.0.1/32\",\"10.0.0.2/32\"]}")
    }

    #[tokio::test]
    async fn delete_peer_returns_404_for_non_existing_dev() {
        let wg = test_helpers::create_wg_mngr(true, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::delete_peer(sync_wg);

        let resp = request()
            .path("/devices/test0/peers/pubkey0")
            .method("DELETE")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn delete_peer_returns_204_for_successful_deletion() {
        let wg = test_helpers::create_wg_mngr(false, false, false);
        let sync_wg = test_helpers::sync_mngr(wg);

        let filter = server::delete_peer(sync_wg);

        let resp = request()
            .path("/devices/test0/peers/pubkey0")
            .method("DELETE")
            .reply(&filter)
            .await;

        assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    }
}
