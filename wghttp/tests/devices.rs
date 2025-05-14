use actix_web::{App, test, web};
use domain::models::netdev::*;
use domain::models::wg::*;
use wghttp::models::devices::*;
use wghttp::models::errors::*;
use wghttp::routes::devices::*;
use wghttp::services::TunnelManager;

use std::net::Ipv4Addr;

pub mod mock;

use mock::*;

#[actix_web::test]
async fn test_list_devices_route_with_empty_result() {
    let wg_mock =
        WireguardMockAdapter::new(None, Some(|| Ok(vec![])), None, None, None, None, None);

    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(list_devices),
    )
    .await;

    let req = test::TestRequest::get().uri("/devices").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(resp.status(), 200);
    let body: Vec<ListDeviceResponse> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 0);
}

#[actix_web::test]
async fn test_list_devices_route_with_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        Some(|| {
            Ok(vec![WGDevice {
                name: "wg0".to_string(),
                public_key: "public_key".to_string(),
                private_key: "private_key".to_string(),
                port: 51820,
                peers: 0,
            }])
        }),
        None,
        None,
        None,
        None,
        None,
    );

    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(list_devices),
    )
    .await;

    let req = test::TestRequest::get().uri("/devices").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(resp.status(), 200);
    let body: Vec<ListDeviceResponse> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0].device_name, "wg0");
    assert_eq!(body[0].port, 51820);
    assert_eq!(body[0].peers, 0);
}

#[actix_web::test]
async fn test_list_devices_route_with_error() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        Some(|| Err(WGError("error".to_string()))),
        None,
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(list_devices),
    )
    .await;

    let req = test::TestRequest::get().uri("/devices").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_server_error());
    assert_eq!(resp.status(), 500);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "error");
}

#[actix_web::test]
async fn test_create_device_route_with_device_name_validation_error() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name_16ch".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: None,
                ipv6: None,
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device name must be at most 15 characters");
}

#[actix_web::test]
async fn test_create_device_route_with_ip_validation_error() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: None,
                ipv6: None,
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(
        body.message,
        "you must provide at least one of ipv4 or ipv6"
    );
}

#[actix_web::test]
async fn test_create_device_route_with_invalid_ip_string() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: Some("invalid_ip".to_string()),
                ipv6: None,
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "invalid ip address: invalid_ip");

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: None,
                ipv6: Some("invalid_ip".to_string()),
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "invalid ip address: invalid_ip");
}

#[actix_web::test]
async fn test_create_device_route_with_wg_fail_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        Some(|_, _| Err(WGError("wg error".to_owned()))),
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: Some("10.0.0.2/32".to_string()),
                ipv6: None,
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 409);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "wg error");
}

#[actix_web::test]
async fn test_create_device_route_with_netdev_set_ip_fail_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        Some(|_, _| {
            Ok(WGDevice {
                name: "name".to_owned(),
                public_key: "pubkey".to_owned(),
                private_key: "privkey".to_owned(),
                port: 51820,
                peers: 2,
            })
        }),
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(
        None,
        Some(|_, _| Err(NetDevError("netdev error".to_owned()))),
        None,
    );
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: Some("10.0.0.2/32".to_string()),
                ipv6: None,
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "netdev error");
}

#[actix_web::test]
async fn test_create_device_route_with_netdev_up_fail_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        Some(|_, _| {
            Ok(WGDevice {
                name: "name".to_owned(),
                public_key: "pubkey".to_owned(),
                private_key: "privkey".to_owned(),
                port: 51820,
                peers: 2,
            })
        }),
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(
        None,
        Some(|_, _| Ok(())),
        Some(|_| Err(NetDevError("netdev error".to_owned()))),
    );
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: Some("10.0.0.2/32".to_string()),
                ipv6: None,
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "netdev error");
}

#[actix_web::test]
async fn test_create_device_route_with_successful_result() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        Some(|n, p| {
            Ok(WGDevice {
                name: n.to_owned(),
                public_key: "pubkey".to_owned(),
                private_key: "privkey".to_owned(),
                port: p,
                peers: 0,
            })
        }),
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, Some(|_, _| Ok(())), Some(|_| Ok(())));
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(create_device),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/devices")
        .set_json(CreateDeviceRequest {
            device_name: "device_name".to_string(),
            port: 51820,
            ip_addresses: DeviceIpAddr {
                ipv4: Some("10.0.0.2/32".to_string()),
                ipv6: Some("2001:db8::2/128".to_string()),
            },
        })
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(resp.status(), 201);
    let body: CreateDeviceResponse = test::read_body_json(resp).await;
    assert_eq!(body.device_name, "device_name");
    assert_eq!(body.port, 51820);
}

#[actix_web::test]
async fn test_get_device_route_with_validation_error() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(get_device),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/devices/invalid_device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device name must be at most 15 characters");
}

#[actix_web::test]
async fn test_get_device_route_with_not_found_error() {
    let wg_mock = WireguardMockAdapter::new(
        Some(|_| Err(WGError("device not found".to_owned()))),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(get_device),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 404);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device not found");
}

#[actix_web::test]
async fn test_get_device_route_with_internal_server_error() {
    let wg_mock = WireguardMockAdapter::new(
        Some(|n| {
            Ok(WGDevice {
                name: n.to_owned(),
                public_key: "pubkey".to_owned(),
                private_key: "privkey".to_owned(),
                port: 51820,
                peers: 0,
            })
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(
        Some(|_| Err(NetDevError("netdev get_ip error".to_owned()))),
        None,
        None,
    );
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(get_device),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_server_error());
    assert_eq!(resp.status(), 500);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "netdev get_ip error");
}

#[actix_web::test]
async fn test_get_device_route_with_successful_result() {
    let wg_mock = WireguardMockAdapter::new(
        Some(|n| {
            Ok(WGDevice {
                name: n.to_owned(),
                public_key: "pubkey".to_owned(),
                private_key: "privkey".to_owned(),
                port: 51820,
                peers: 0,
            })
        }),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(
        Some(|_| {
            Ok(NetDevIp {
                ipv4: Some((Ipv4Addr::new(10, 0, 0, 2), 32)),
                ipv6: None,
            })
        }),
        None,
        None,
    );
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(get_device),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/devices/device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(resp.status(), 200);
    let body: DetailDeviceResponse = test::read_body_json(resp).await;
    assert_eq!(body.device_name, "device_name");
    assert_eq!(body.port, 51820);
    assert_eq!(body.ip_addresses.ipv4, Some("10.0.0.2/32".to_string()));
    assert_eq!(body.ip_addresses.ipv6, None);
    assert_eq!(body.public_key, "pubkey");
    assert_eq!(body.peers, 0);
}

#[actix_web::test]
async fn test_delete_device_route_with_validation_error() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, None, None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(delete_device),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/devices/invalid_device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 400);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device name must be at most 15 characters");
}

#[actix_web::test]
async fn test_delete_device_route_with_not_found_error() {
    let wg_mock = WireguardMockAdapter::new(
        None,
        None,
        None,
        Some(|_| Err(WGError("device not found".to_owned()))),
        None,
        None,
        None,
    );
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(delete_device),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/devices/device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_client_error());
    assert_eq!(resp.status(), 404);
    let body: Error = test::read_body_json(resp).await;
    assert_eq!(body.message, "device not found");
}

#[actix_web::test]
async fn test_delete_device_route_with_successful_result() {
    let wg_mock = WireguardMockAdapter::new(None, None, None, Some(|_| Ok(())), None, None, None);
    let netdev_mock = NetworkDeviceMockAdapter::new(None, None, None);
    let tunnel_manager = TunnelManager::new(wg_mock, netdev_mock);

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
            .service(delete_device),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/devices/device_name")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(resp.status(), 204);
}
