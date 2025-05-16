use actix_web::{HttpResponse, Responder, get};

#[utoipa::path(
    get,
    path = "/",
    tag = "health",
    responses(
        (status = 200, description = "service is healthy"),
        (status = 503, description = "service is unhealthy")
    )
)]
#[get("/")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
}
