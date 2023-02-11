pub mod device {}

pub mod peers {}

pub mod server {
    use std::net::SocketAddr;

    use utoipa::OpenApi;
    use warp::Filter;

    use crate::api::models::*;

    #[utoipa::path(
        get,
        path = "/_health",
        tag = "health",
        responses(
            (status = 200, description = "service is working well"),
            (status = 503, description = "service is not working healthy")
        )
    )]
    pub fn health() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("_health")
            .and(warp::get())
            .map(|| warp::reply::reply())
    }

    #[utoipa::path(
        get,
        path = "/devices",
        tag = "devices",
        responses(
            (status = 200, description = "successful listing", body = Vec<ListDevice>),
        )
    )]
    pub async fn list_devices() {}

    #[utoipa::path(
        post,
        path = "/devices",
        request_body = CreateDevice,
        tag = "devices",
        responses(
            (status = 201, description = "successful creating", body = DetailDevice),
            (status = 400, description = "validation error", body = Error),
            (status = 409, description = "conflict error", body = Error),
        )
    )]
    pub async fn create_device() {}

    #[utoipa::path(
        get,
        path = "/devices/{device_name}",
        tag = "devices",
        responses(
            (status = 200, description = "successful detail", body = DetailDevice),
            (status = 404, description = "device not found")
        ),
        params(
            ("device_name" = String, Path, description = "wireguard device name")
        )
    )]
    pub async fn get_device() {}

    #[utoipa::path(
        delete,
        path = "/devices/{device_name}",
        tag = "devices",
        responses(
            (status = 204, description = "successful delete"),
            (status = 404, description = "device not found")
        ),
        params(
            ("device_name" = String, Path, description = "wireguard device name")
        )
    )]
    pub async fn delete_device() {}

    #[utoipa::path(
        get,
        path = "/devices/{device_name}/peers",
        tag = "peers",
        responses(
            (status = 200, description = "successful peers listing", body = Vec<ListPeer>),
            (status = 404, description = "device not found")
        ),
        params(
            ("device_name" = String, Path, description = "wireguard device name")
        )
    )]
    pub async fn list_peers() {}

    #[utoipa::path(
        post,
        path = "/devices/{device_name}/peers",
        request_body = CreatePeer,
        tag = "peers",
        responses(
            (status = 200, description = "successful peers creating", body = DetailPeer),
            (status = 400, description = "validation error", body = Error),
            (status = 404, description = "device not found")
        ),
        params(
            ("device_name" = String, Path, description = "wireguard device name")
        )
    )]
    pub async fn create_peer() {}

    #[utoipa::path(
        delete,
        path = "/devices/{device_name}/peers/{public_key}",
        tag = "peers",
        responses(
            (status = 204, description = "successful delete"),
            (status = 404, description = "device or peer not found")
        ),
        params(
            ("device_name" = String, Path, description = "wireguard device name"),
            ("public_key" = String, Path, description = "peer's public key")
        )
    )]
    pub async fn delete_peer() {}

    #[derive(OpenApi)]
    #[openapi(
        paths(
            health,
            list_devices,
            create_device,
            get_device,
            delete_device,
            list_peers,
            create_peer,
            delete_peer,
        ),
        components(schemas(
            Error,
            ListDevice,
            CreateDevice,
            DetailDevice,
            ListPeer,
            CreatePeer,
            DetailPeer,
        ))
    )]
    struct ApiDoc;

    fn openapi_doc() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("api-doc.json")
            .and(warp::get())
            .map(|| warp::reply::json(&ApiDoc::openapi()))
    }

    pub async fn serve(addr: impl Into<SocketAddr>) {
        let routes = health().or(openapi_doc());
        warp::serve(routes).run(addr).await
    }
}
