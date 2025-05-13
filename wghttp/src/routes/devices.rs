use crate::helpers::*;
use crate::models::devices::*;
use crate::models::errors::Error;
use crate::services::TunnelManager;
use actix_web::{HttpResponse, Responder, delete, get, post, web};
use domain::models::netdev::NetDevIp;

#[utoipa::path(
    get,
    path = "/devices",
    tag = "devices",
    responses(
        (status = 200, description = "list of wireguard devices", body = [ListDeviceResponse])
    )
)]
#[get("/devices")]
async fn list_devices(tm: web::Data<TunnelManager>) -> impl Responder {
    let manager = tm.get_ref();
    let devices = manager.wireguard.list_devices();
    match devices {
        Err(e) => HttpResponse::InternalServerError().json(Error { message: e.0 }),
        Ok(wgdevs) => {
            let out: Vec<ListDeviceResponse> = wgdevs
                .into_iter()
                .map(|d| ListDeviceResponse {
                    device_name: d.name,
                    port: d.port,
                    peers: d.peers,
                })
                .collect();
            HttpResponse::Ok().json(out)
        }
    }
}

#[utoipa::path(
    post,
    path = "/devices",
    tag = "devices",
    request_body = CreateDeviceRequest,
    responses(
        (status = 201, description = "device created successfully", body = CreateDeviceResponse),
        (status = 400, description = "validation error", body = Error),
        (status = 409, description = "conflict error", body = Error),
        (status = 500, description = "system error", body = Error),
    )
)]
#[post("/devices")]
async fn create_device(
    tm: web::Data<TunnelManager>,
    device: web::Json<CreateDeviceRequest>,
) -> impl Responder {
    if device.device_name.len() > DEVICE_NAME_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "device name must be at most 15 characters".to_owned(),
        });
    }

    if device.ip_addresses.ipv4.is_none() && device.ip_addresses.ipv6.is_none() {
        return HttpResponse::BadRequest().json(Error {
            message: "you must provide at least one of ipv4 or ipv6".to_owned(),
        });
    }

    let parsed_ipv4 = device
        .ip_addresses
        .ipv4
        .as_ref()
        .map(|s| parse_ip(s))
        .transpose()
        .map_err(|e| Error { message: e });

    let ipv4 = match parsed_ipv4 {
        Ok(opt_ip) => opt_ip,
        Err(e) => return HttpResponse::BadRequest().json(e),
    };

    let parsed_ipv6 = device
        .ip_addresses
        .ipv6
        .as_ref()
        .map(|s| parse_ip(s))
        .transpose()
        .map_err(|e| Error { message: e });

    let ipv6 = match parsed_ipv6 {
        Ok(opt_ip) => opt_ip,
        Err(e) => return HttpResponse::BadRequest().json(e),
    };

    let manager = tm.get_ref();

    let wg_result = manager
        .wireguard
        .create_device(&device.device_name, device.port);
    if let Err(e) = wg_result {
        return HttpResponse::Conflict().json(Error { message: e.0 });
    }

    let ip = NetDevIp::new(ipv4, ipv6);
    let netdev_result = manager
        .netdev
        .set_ip(&device.device_name, &ip)
        .and_then(|_| manager.netdev.up(&device.device_name));
    if let Err(e) = netdev_result {
        return HttpResponse::BadRequest().json(Error { message: e.0 });
    }

    let Ok(d) = wg_result else {
        return HttpResponse::InternalServerError().json(Error {
            message: "system error".to_string(),
        });
    };

    let dev = CreateDeviceResponse {
        device_name: d.name,
        port: d.port,
        ip_addresses: DeviceIpAddr {
            ipv4: ip.ipv4_str(),
            ipv6: ip.ipv6_str(),
        },
        private_key: d.private_key,
        public_key: d.public_key,
    };
    HttpResponse::Created().json(dev)
}

#[utoipa::path(
    get,
    path = "/devices/{dev}",
    tag = "devices",
    params(
        ("dev", description = "device name")
    ),
    responses(
        (status = 200, description = "device found", body = DetailDeviceResponse),
        (status = 400, description = "validation error", body = Error),
        (status = 404, description = "device not found"),
        (status = 500, description = "system error", body = Error),
    )
)]
#[get("/devices/{dev}")]
async fn get_device(tm: web::Data<TunnelManager>, path: web::Path<String>) -> impl Responder {
    let dev_name = path.into_inner();
    if dev_name.len() > DEVICE_NAME_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "device name must be at most 15 characters".to_owned(),
        });
    }

    let manager = tm.get_ref();
    let wg_result = manager.wireguard.get_device(&dev_name);
    if let Err(e) = wg_result {
        return HttpResponse::NotFound().json(Error { message: e.0 });
    }

    let netdev_result = manager.netdev.get_ip(&dev_name);
    if let Err(e) = netdev_result {
        return HttpResponse::InternalServerError().json(Error { message: e.0 });
    }

    let (Ok(d), Ok(ip)) = (wg_result, netdev_result) else {
        return HttpResponse::InternalServerError().json(Error {
            message: "system error".to_string(),
        });
    };

    let out = DetailDeviceResponse {
        device_name: d.name,
        port: d.port,
        ip_addresses: DeviceIpAddr {
            ipv4: ip.ipv4_str(),
            ipv6: ip.ipv6_str(),
        },
        public_key: d.public_key,
        peers: d.peers,
    };
    HttpResponse::Ok().json(out)
}

#[utoipa::path(
    delete,
    path = "/devices/{dev}",
    tag = "devices",
    params(
        ("dev", description = "device name")
    ),
    responses(
        (status = 204, description = "successfully deleted"),
        (status = 400, description = "validation error", body = Error),
        (status = 404, description = "device not found")
    )
)]
#[delete("/devices/{dev}")]
async fn delete_device(tm: web::Data<TunnelManager>, path: web::Path<String>) -> impl Responder {
    let dev_name = path.into_inner();
    if dev_name.len() > DEVICE_NAME_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "device name must be at most 15 characters".to_owned(),
        });
    }

    let manager = tm.get_ref();
    match manager.wireguard.delete_device(&dev_name) {
        Err(e) => HttpResponse::NotFound().json(Error { message: e.0 }),
        Ok(()) => HttpResponse::NoContent().finish(),
    }
}
