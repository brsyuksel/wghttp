use actix_web::{HttpResponse, Responder, delete, get, post, web};
use crate::models::peers::*;
use crate::models::errors::Error;

#[utoipa::path(
    get,
    path = "/devices/{dev}/peers",
    tag = "peers",
    params(
        ("dev", description = "device name")
    ),
    responses(
        (status = 200, description = "list of peers for given wireguard device", body = [ListPeerResponse]),
        (status = 404, description = "device not found")
    )
)]
#[get("/devices/{dev}/peers")]
async fn list_peers(path: web::Path<(String, )>) -> impl Responder {
    let (dev, ) = path.into_inner();
    println!("device name is {}", dev);
    HttpResponse::Ok()
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
async fn create_peer(path: web::Path<(String, )>, peer: web::Json<CreatePeerRequest>) -> impl Responder {
    let (dev, ) = path.into_inner();
    println!("device name is {}", dev);
    HttpResponse::Created()
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
        (status = 404, description = "device or peer not found")
    )
)]
#[delete("/devices/{dev}/peers/{public_key}")]
async fn delete_peer(path: web::Path<(String, String)>) -> impl Responder {
    let (dev, public_key) = path.into_inner();
    println!("device name is {} and public_key is {}", dev, public_key);
    HttpResponse::Ok()
}
