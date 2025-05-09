use crate::models::devices::*;
use crate::models::errors::Error;
use crate::services::TunnelManager;
use actix_web::{HttpResponse, Responder, delete, get, post, web};

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
    )
)]
#[post("/devices")]
async fn create_device(
    tm: web::Data<TunnelManager>,
    device: web::Json<CreateDeviceRequest>,
) -> impl Responder {
    // TODO: input validation

    let manager = tm.get_ref();
    let result = manager
        .wireguard
        .create_device(&device.device_name, device.port);
    match result {
        Err(e) => HttpResponse::Conflict().json(Error { message: e.0 }),
        Ok(d) => {
            let dev = CreateDeviceResponse {
                device_name: d.name,
                port: d.port,
                ip_addresses: DeviceIpAddr {
                    ipv4: None,
                    ipv6: None,
                },
                private_key: d.private_key,
                public_key: d.public_key,
            };
            HttpResponse::Created().json(dev)
        }
    }
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
        (status = 404, description = "device not found")
    )
)]
#[get("/devices/{dev}")]
async fn get_device(tm: web::Data<TunnelManager>, path: web::Path<String>) -> impl Responder {
    let dev_name = path.into_inner();
    let manager = tm.get_ref();
    let result = manager.wireguard.get_device(&dev_name);
    match result {
        Err(e) => HttpResponse::NotFound().json(Error { message: e.0 }),
        Ok(d) => {
            let out = DetailDeviceResponse {
                device_name: d.name,
                port: d.port,
                ip_addresses: DeviceIpAddr {
                    ipv4: None,
                    ipv6: None,
                },
                public_key: d.public_key,
                peers: d.peers,
            };
            HttpResponse::Ok().json(out)
        }
    }
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
        (status = 404, description = "device not found")
    )
)]
#[delete("/devices/{dev}")]
async fn delete_device(tm: web::Data<TunnelManager>, path: web::Path<String>) -> impl Responder {
    let dev_name = path.into_inner();
    let manager = tm.get_ref();
    match manager.wireguard.delete_device(&dev_name) {
        Err(e) => HttpResponse::NotFound().json(Error { message: e.0 }),
        Ok(()) => HttpResponse::NoContent().finish(),
    }
}
