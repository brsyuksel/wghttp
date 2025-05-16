use actix_web::{App, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use clap::Parser;

use wghttp::*;

use netdev::NetDevAdapter;
use wgshim::WGShimAdapter;

/// wghttp - HTTP service to manage wireguard devices
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// path to unix socket. it is default unless you provide tcp socket
    #[clap(short, long, default_value = "/var/run/wghttp.sock")]
    unix: String,

    /// tcp socket to listen on (optional)
    #[clap(short, long)]
    tcp: Option<String>,
}

impl Args {
    fn is_unix(&self) -> bool {
        self.tcp.is_none()
    }

    fn host_and_port(&self) -> Option<(String, u16)> {
        if let Some(tcp) = &self.tcp {
            let parts: Vec<&str> = tcp.split(':').collect();
            if parts.len() == 2 {
                if let Ok(port) = parts[1].parse::<u16>() {
                    return Some((parts[0].to_string(), port));
                }
            }
        }
        None
    }

    fn unix_path(&self) -> String {
        self.unix.clone()
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

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

    let tunnel_manager = services::TunnelManager::new(WGShimAdapter, NetDevAdapter);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tunnel_manager.clone()))
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
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    });
    
    let bound = if args.is_unix() {
        server.bind_uds(&args.unix_path())
    } else {
        let (host, port) = args.host_and_port().expect("invalid tcp socket");
        server.bind((host.as_str(), port))
    };

    bound?.run().await
}
