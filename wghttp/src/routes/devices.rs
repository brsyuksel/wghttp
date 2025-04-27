use actix_web::{delete, get, post, web, HttpResponse, Responder};
use crate::models::devices::*;
use crate::models::errors::Error;

#[utoipa::path(
    get,
    path = "/devices",
    tag = "devices",
    responses(
        (status = 200, description = "list of wireguard devices", body = [ListDeviceResponse])
    )
)]
#[get("/devices")]
async fn list_devices() -> impl Responder {
    HttpResponse::Ok()
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
async fn create_device(device: web::Json<CreateDeviceRequest>) -> impl Responder {
    HttpResponse::Created()
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
async fn get_device(path: web::Path<String>) -> impl Responder {
    let dev = path.into_inner();
    println!("device name is {}", dev);
    HttpResponse::Ok()
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
async fn delete_device(path: web::Path<String>) -> impl Responder {
    let dev = path.into_inner();
    println!("device name is {}", dev);
    HttpResponse::Ok()
}
