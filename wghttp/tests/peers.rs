use actix_web::{App, test, web};
use domain::models::wg::*;
use wghttp::models::errors::*;
use wghttp::models::peers::*;
use wghttp::routes::peers::*;
use wghttp::services::TunnelManager;

pub mod mock;

use mock::*;

#[actix_web::test]
async fn test_list_peers_route_with_validation_error() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app = test::init_service(App::new().app_data(web::Data::new(tm)).service(list_peers)).await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name_16ch/peers")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device name must be at most 15 characters");
}

#[actix_web::test]
async fn test_list_peers_route_with_not_found() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        None,
        None,
        Some(|_| Err(WGError("device not found".to_owned()))),
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app = test::init_service(App::new().app_data(web::Data::new(tm)).service(list_peers)).await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name/peers")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 404);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device not found");
}

#[actix_web::test]
async fn test_list_peers_route_with_empty_success_result() {
    let wg_mock =
        WireguardMockAdapter::new(None, None, None, None, Some(|_| Ok(vec![])), None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app = test::init_service(App::new().app_data(web::Data::new(tm)).service(list_peers)).await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name/peers")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_success(), true);
    assert_eq!(resp.status(), 200);
    let body: Vec<ListPeerResponse> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 0);
}

#[actix_web::test]
async fn test_list_peers_route_with_success_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        None,
        None,
        Some(|_| {
            Ok(vec![WGPeer {
                public_key: "public_key".to_owned(),
                private_key: "private_key".to_owned(),
                preshared_key: "preshared_key".to_owned(),
                endpoint: "endpoint".to_owned(),
                allowed_ips: vec!["allowed_ips".to_owned()],
                last_handshake_time: 0,
                persistent_keepalive_interval: 0,
                rx: 0,
                tx: 0,
            }])
        }),
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app = test::init_service(App::new().app_data(web::Data::new(tm)).service(list_peers)).await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name/peers")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_success(), true);
    assert_eq!(resp.status(), 200);
    let body: Vec<ListPeerResponse> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].public_key, "public_key");
    assert_eq!(body[0].endpoint, "endpoint");
    assert_eq!(body[0].allowed_ips, vec!["allowed_ips".to_owned()]);
    assert_eq!(body[0].last_handshake_time, 0);
    assert_eq!(body[0].persistent_keepalive_interval, 0);
}

#[actix_web::test]
async fn test_create_peer_route_with_validation_error() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(create_peer)).await;

    let req = test::TestRequest::post()
        .uri("/devices/device_name_16ch/peers")
        .set_json(CreatePeerRequest {
            allowed_ips: vec!["allowed_ips".to_owned()],
            persistent_keepalive_interval: 0,
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device name must be at most 15 characters");
}

#[actix_web::test]
async fn test_create_peer_route_with_invalid_ip_string() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(create_peer)).await;

    let req = test::TestRequest::post()
        .uri("/devices/device_name/peers")
        .set_json(CreatePeerRequest {
            allowed_ips: vec!["invalid_ip_str".to_owned()],
            persistent_keepalive_interval: 0,
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "invalid ip address: invalid_ip_str");
}

#[actix_web::test]
async fn test_create_peer_route_with_not_found() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        None,
        None,
        None,
        Some(|_, _, _| Err(WGError("device not found".to_owned()))),
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(create_peer)).await;

    let req = test::TestRequest::post()
        .uri("/devices/device_name/peers")
        .set_json(CreatePeerRequest {
            allowed_ips: vec!["10.0.0.2/32".to_owned()],
            persistent_keepalive_interval: 0,
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 404);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device not found");
}

#[actix_web::test]
async fn test_create_peer_route_with_successful_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        None,
        None,
        None,
        Some(|_, i, p| {
            Ok(WGPeer {
                allowed_ips: i.into_iter().map(|s| s.to_owned()).collect(),
                endpoint: "endpoint".to_owned(),
                last_handshake_time: 0,
                persistent_keepalive_interval: p,
                rx: 0,
                tx: 0,
                public_key: "pubkey".to_owned(),
                private_key: "privkey".to_owned(),
                preshared_key: "preshared".to_owned(),
            })
        }),
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(create_peer)).await;

    let req = test::TestRequest::post()
        .uri("/devices/device_name/peers")
        .set_json(CreatePeerRequest {
            allowed_ips: vec!["10.0.0.2/32".to_owned()],
            persistent_keepalive_interval: 30,
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_success(), true);
    assert_eq!(resp.status(), 201);
    let body: CreatePeerResponse = test::read_body_json(resp).await;
    assert_eq!(body.public_key, "pubkey");
    assert_eq!(body.private_key, "privkey");
    assert_eq!(body.preshared_key, "preshared");
    assert_eq!(body.allowed_ips, vec!["10.0.0.2/32".to_owned()]);
    assert_eq!(body.persistent_keepalive_interval, 30);
}

#[actix_web::test]
async fn test_delete_peer_route_with_validation_error_dev_name() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(delete_peer)).await;

    let req = test::TestRequest::delete()
        .uri("/devices/device_name_16ch/peers/public_key")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device name must be at most 15 characters");
}

#[actix_web::test]
async fn test_delete_peer_route_with_validation_error_pub_key() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(delete_peer)).await;

    let req = test::TestRequest::delete()
        .uri("/devices/device_name/peers/pubkeypubkeypubkeypubkeypubkeypubkeypubkeypub")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "public key must be 44 characters");
}

#[actix_web::test]
async fn test_delete_peer_route_with_not_found() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        None,
        None,
        None,
        None,
        Some(|_, _| Err(WGError("peer not found".to_owned()))),
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(delete_peer)).await;

    let req = test::TestRequest::delete()
        .uri("/devices/device_name/peers/pubkeypubkeypubkeypubkeypubkeypubkeypubkeypu")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_client_error(), true);
    assert_eq!(resp.status(), 404);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "peer not found");
}

#[actix_web::test]
async fn test_delete_peer_route_with_successful_result() {
    let wg_mock =
        WireguardMockAdapter::new(None, None, None, None, None, None, Some(|_, _| Ok(())));
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tm = TunnelManager::new(wg_mock, netdev_mock);
    let app =
        test::init_service(App::new().app_data(web::Data::new(tm)).service(delete_peer)).await;

    let req = test::TestRequest::delete()
        .uri("/devices/device_name/peers/pubkeypubkeypubkeypubkeypubkeypubkeypubkeypu")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status().is_success(), true);
    assert_eq!(resp.status(), 204);
}
