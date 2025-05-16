use crate::helpers::*;
use crate::models::errors::Error;
use crate::models::peers::*;
use crate::services::TunnelManager;
use actix_web::{HttpResponse, Responder, delete, get, post, web};

#[utoipa::path(
    get,
    path = "/devices/{dev}/peers",
    tag = "peers",
    params(
        ("dev", description = "device name")
    ),
    responses(
        (status = 200, description = "list of peers for given wireguard device", body = [ListPeerResponse]),
        (status = 400, description = "validation error", body = Error),
        (status = 404, description = "device not found")
    )
)]
#[get("/devices/{dev}/peers")]
async fn list_peers(tm: web::Data<TunnelManager>, path: web::Path<String>) -> impl Responder {
    let dev_name = path.into_inner();
    if dev_name.len() > DEVICE_NAME_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "device name must be at most 15 characters".to_owned(),
        });
    }

    let manager = tm.get_ref();
    let peers = manager.wireguard.list_peers(&dev_name);
    match peers {
        Err(e) => HttpResponse::NotFound().json(Error { message: e.0 }),
        Ok(wgpeers) => {
            let out: Vec<ListPeerResponse> = wgpeers
                .into_iter()
                .map(|p| ListPeerResponse {
                    public_key: p.public_key,
                    endpoint: p.endpoint,
                    allowed_ips: p.allowed_ips,
                    last_handshake_time: p.last_handshake_time,
                    persistent_keepalive_interval: p.persistent_keepalive_interval,
                    rx: p.rx,
                    tx: p.tx,
                })
                .collect();
            HttpResponse::Ok().json(out)
        }
    }
}

#[utoipa::path(
    post,
    path = "/devices/{dev}/peers",
    tag = "peers",
    params(
        ("dev", description = "device name")
    ),
    request_body = CreatePeerRequest,
    responses(
        (status = 201, description = "peer created successfully", body = CreatePeerResponse),
        (status = 400, description = "validation error", body = Error),
        (status = 404, description = "device not found"),
    )
)]
#[post("/devices/{dev}/peers")]
async fn create_peer(
    tm: web::Data<TunnelManager>,
    path: web::Path<String>,
    peer: web::Json<CreatePeerRequest>,
) -> impl Responder {
    let dev_name = path.into_inner();
    if dev_name.len() > DEVICE_NAME_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "device name must be at most 15 characters".to_owned(),
        });
    }

    if let Err(e) = validate_ip_list(&peer.allowed_ips) {
        return HttpResponse::BadRequest().json(Error { message: e });
    }

    let manager = tm.get_ref();
    let ips: Vec<&str> = peer.allowed_ips.iter().map(|s| s.as_str()).collect();
    let result = manager
        .wireguard
        .add_peer(&dev_name, ips, peer.persistent_keepalive_interval);
    match result {
        Err(e) => HttpResponse::NotFound().json(Error { message: e.0 }),
        Ok(wgpeer) => {
            let peer = CreatePeerResponse {
                public_key: wgpeer.public_key,
                private_key: wgpeer.private_key,
                preshared_key: wgpeer.preshared_key,
                allowed_ips: wgpeer.allowed_ips,
                persistent_keepalive_interval: wgpeer.persistent_keepalive_interval,
            };
            HttpResponse::Created().json(peer)
        }
    }
}

#[utoipa::path(
    delete,
    path = "/devices/{dev}/peers/{public_key}",
    tag = "peers",
    params(
        ("dev", description = "device name"),
        ("public_key", description = "peer' public key")
    ),
    responses(
        (status = 204, description = "successfully deleted"),
        (status = 400, description = "validation error", body = Error),
        (status = 404, description = "device or peer not found")
    )
)]
#[delete("/devices/{dev}/peers/{public_key}")]
async fn delete_peer(
    tm: web::Data<TunnelManager>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (dev, public_key) = path.into_inner();
    if dev.len() > DEVICE_NAME_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "device name must be at most 15 characters".to_owned(),
        });
    }

    if public_key.len() != PUBKEY_MAX_LEN {
        return HttpResponse::BadRequest().json(Error {
            message: "public key must be 44 characters".to_owned(),
        });
    }

    let manager = tm.get_ref();
    match manager.wireguard.delete_peer(&dev, &public_key) {
        Err(e) => HttpResponse::NotFound().json(Error { message: e.0 }),
        Ok(()) => HttpResponse::NoContent().finish(),
    }
}
