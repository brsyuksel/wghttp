use actix_web::{App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod routes;
mod models;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    #[derive(OpenApi)]
    #[openapi(
        info(
            title = "wghttp",
            description = "http service to manage wireguard devices",
            version = "1.0.0",
            license(name = "MIT")
        ),
        tags(
            (name = "health", description = "health check endpoint."),
            (name = "devices", description = "device management endpoints."),
            (name = "peers", description = "peer management endpoints.")
        ),
        paths(
            routes::health::health,
            routes::devices::list_devices,
            routes::devices::create_device,
            routes::devices::get_device,
            routes::devices::delete_device,
            routes::peers::list_peers,
            routes::peers::create_peer,
            routes::peers::delete_peer,
        )
    )]
    struct ApiDoc;

    HttpServer::new(|| {
        App::new()
            .service(routes::health::health)
            .service(routes::devices::list_devices)
            .service(routes::devices::create_device)
            .service(routes::devices::get_device)
            .service(routes::devices::delete_device)
            .service(routes::peers::list_peers)
            .service(routes::peers::create_peer)
            .service(routes::peers::delete_peer)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                .url("/api-docs/openapi.json", ApiDoc::openapi())
            )
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
