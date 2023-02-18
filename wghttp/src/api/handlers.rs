pub mod server {
    use std::convert::Infallible;
    use std::net::SocketAddr;
    use std::sync::Arc;

    use ipnet::Ipv4Net;
    use serde::de::DeserializeOwned;
    use tokio::sync::Mutex;
    use tokio_stream::wrappers::UnixListenerStream;
    use utoipa::OpenApi;
    use warp::http::StatusCode;
    use warp::reply::json;
    use warp::Filter;

    use basis::if_mng::*;
    use basis::wg_mng::*;

    use crate::api::models::*;

    type SyncMngr<T> = Arc<Mutex<T>>;

    fn with_wg(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> impl Filter<
        Extract = (SyncMngr<dyn WireguardManager + Send>,),
        Error = std::convert::Infallible,
    > + Clone {
        warp::any().map(move || wg.clone())
    }

    fn with_im(
        im: SyncMngr<dyn InterfaceManager + Send>,
    ) -> impl Filter<
        Extract = (SyncMngr<dyn InterfaceManager + Send>,),
        Error = std::convert::Infallible,
    > + Clone {
        warp::any().map(move || im.clone())
    }

    fn with_json_payload<T: DeserializeOwned + Send>(
    ) -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

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
    pub fn list_devices(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices")
            .and(warp::get())
            .and(with_wg(wg))
            .and_then(list_devices_handler)
    }

    async fn list_devices_handler(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        let mngr = wg.lock().await;
        let devices_res = mngr.list_devices();

        if let Err(e) = devices_res {
            let output = Error { message: e.0 };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        let devs = devices_res.unwrap();

        let output = devs
            .iter()
            .map(|wd| ListDevice {
                device_name: wd.name.clone(),
                port: wd.port,
                total_peers: wd.total_peers,
            })
            .collect::<Vec<ListDevice>>();

        Ok(warp::reply::with_status(json(&output), StatusCode::OK))
    }

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
    pub fn create_device(
        wg: SyncMngr<dyn WireguardManager + Send>,
        im: SyncMngr<dyn InterfaceManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices")
            .and(warp::post())
            .and(with_json_payload::<CreateDevice>())
            .and(with_wg(wg))
            .and(with_im(im))
            .and_then(create_device_handler)
    }

    async fn create_device_handler(
        body: CreateDevice,
        wg: SyncMngr<dyn WireguardManager + Send>,
        im: SyncMngr<dyn InterfaceManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        let addr_result = body.ip.parse::<Ipv4Net>();
        if let Err(_) = addr_result {
            let output = Error {
                message: "error on parsing ip addr".to_owned(),
            };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::BAD_REQUEST,
            ));
        }
        let addr = addr_result.unwrap();

        let wg_mngr = wg.lock().await;
        let if_mngr = im.lock().await;

        let added_result = wg_mngr.add_device(&body.device_name, body.port);
        if let Err(added_err) = added_result {
            let output = Error {
                message: added_err.0,
            };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
        let added_dev = added_result.unwrap();

        if let Err(set_ip_err) =
            if_mngr.set_ip_and_netmask(added_dev.name.as_str(), &addr.network(), &addr.netmask())
        {
            let output = Error {
                message: set_ip_err.0,
            };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::CONFLICT,
            ));
        }

        if let Err(up_err) = if_mngr.up_device(added_dev.name.as_str()) {
            let output = Error { message: up_err.0 };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::CONFLICT,
            ));
        }

        let output = DetailDevice {
            device_name: added_dev.name,
            port: added_dev.port,
            ip: body.ip,
            public_key: added_dev.public_key,
            private_key: added_dev.private_key,
            total_peers: added_dev.total_peers,
        };

        Ok(warp::reply::with_status(json(&output), StatusCode::CREATED))
    }

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
    pub fn get_device(
        wg: SyncMngr<dyn WireguardManager + Send>,
        im: SyncMngr<dyn InterfaceManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices" / String)
            .and(warp::get())
            .and(with_wg(wg))
            .and(with_im(im))
            .and_then(get_device_handler)
    }

    async fn get_device_handler(
        device_name: String,
        wg: SyncMngr<dyn WireguardManager + Send>,
        im: SyncMngr<dyn InterfaceManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        let wg_mngr = wg.lock().await;

        let dev_result = wg_mngr.get_device(device_name.as_str());
        if let Err(dev_err) = dev_result {
            let output = Error { message: dev_err.0 };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::NOT_FOUND,
            ));
        }
        let dev = dev_result.unwrap();

        let output = DetailDevice {
            device_name: dev.name,
            port: dev.port,
            ip: "".to_owned(), // TODO
            public_key: dev.public_key,
            private_key: dev.private_key,
            total_peers: dev.total_peers,
        };

        Ok(warp::reply::with_status(json(&output), StatusCode::OK))
    }

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
    pub fn delete_device(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices" / String)
            .and(warp::delete())
            .and(with_wg(wg))
            .and_then(delete_device_handler)
    }

    async fn delete_device_handler(
        device_name: String,
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        let wg_mngr = wg.lock().await;

        let status_code = match wg_mngr.del_device(device_name.as_str()) {
            Err(_) => StatusCode::NOT_FOUND,
            Ok(_) => StatusCode::NO_CONTENT,
        };

        Ok(status_code)
    }

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
    pub fn list_peers(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices" / String / "peers")
            .and(warp::get())
            .and(with_wg(wg))
            .and_then(list_peers_handler)
    }

    async fn list_peers_handler(
        device_name: String,
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        let wg_mngr = wg.lock().await;

        let peers_res = wg_mngr.list_peers(device_name.as_str());
        if let Err(e) = peers_res {
            let output = Error { message: e.0 };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::NOT_FOUND,
            ));
        }

        let peers = peers_res.unwrap();

        let output = peers
            .iter()
            .map(|p| ListPeer {
                public_key: p.public_key.clone(),
                endpoint: p.endpoint.clone(),
                last_handshake_time: p.last_handshake_time as u64, // FIXME
                rx: p.rx,
                tx: p.tx,
                persistent_keepalive_time: p.persistent_keepalive_interval,
                allowed_ips: p.allowed_ips.clone(),
            })
            .collect::<Vec<ListPeer>>();

        Ok(warp::reply::with_status(json(&output), StatusCode::OK))
    }

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
    pub fn create_peer(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices" / String / "peers")
            .and(warp::post())
            .and(with_json_payload::<CreatePeer>())
            .and(with_wg(wg))
            .and_then(create_peer_handler)
    }

    async fn create_peer_handler(
        device_name: String,
        body: CreatePeer,
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        for addr in body.allowed_ips.iter() {
            let parse_result = addr.parse::<Ipv4Net>();
            if parse_result.is_err() {
                let output = Error {
                    message: "error on parsing ip addr".to_owned(),
                };
                return Ok(warp::reply::with_status(
                    json(&output),
                    StatusCode::BAD_REQUEST,
                ));
            }
        }

        let wg_mngr = wg.lock().await;

        let add_peer_res = wg_mngr.add_peer(
            device_name.as_str(),
            body.allowed_ips.clone(),
            body.persistent_keepalive_time,
        );
        if let Err(e) = add_peer_res {
            let output = Error { message: e.0 };
            return Ok(warp::reply::with_status(
                json(&output),
                StatusCode::NOT_FOUND,
            ));
        }

        let (peer, priv_key) = add_peer_res.unwrap();
        let output = DetailPeer {
            public_key: peer.public_key,
            private_key: priv_key,
            endpoint: peer.endpoint,
            last_handshake_time: peer.last_handshake_time as u64,
            rx: peer.rx,
            tx: peer.tx,
            persistent_keepalive_time: peer.persistent_keepalive_interval,
            allowed_ips: peer.allowed_ips.clone(),
        };
        Ok(warp::reply::with_status(json(&output), StatusCode::OK))
    }

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
    pub fn delete_peer(
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("devices" / String / "peers" / String)
            .and(warp::delete())
            .and(with_wg(wg))
            .and_then(delete_peer_handler)
    }

    async fn delete_peer_handler(
        device_name: String,
        public_key: String,
        wg: SyncMngr<dyn WireguardManager + Send>,
    ) -> Result<impl warp::Reply, Infallible> {
        let wg_mngr = wg.lock().await;

        let status_code = match wg_mngr.del_peer(device_name.as_str(), public_key.as_str()) {
            Err(_) => StatusCode::NOT_FOUND,
            Ok(_) => StatusCode::NO_CONTENT,
        };

        Ok(status_code)
    }

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

    fn get_routes<I, W>(
        if_mngr: SyncMngr<I>,
        wg_mngr: SyncMngr<W>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    where
        I: InterfaceManager + Send + 'static,
        W: WireguardManager + Send + 'static,
    {
        health()
            .or(openapi_doc())
            .or(list_devices(wg_mngr.clone()))
            .or(create_device(wg_mngr.clone(), if_mngr.clone()))
            .or(get_device(wg_mngr.clone(), if_mngr))
            .or(delete_device(wg_mngr.clone()))
            .or(list_peers(wg_mngr.clone()))
            .or(create_peer(wg_mngr.clone()))
            .or(delete_peer(wg_mngr))
    }

    pub async fn serve_tcp<I, W>(addr: impl Into<SocketAddr>, if_mngr: I, wg_mngr: W)
    where
        I: InterfaceManager + Send + 'static,
        W: WireguardManager + Send + 'static,
    {
        let sync_if_mngr = Arc::new(Mutex::new(if_mngr));
        let sync_wg_mngr = Arc::new(Mutex::new(wg_mngr));

        let routes = get_routes(sync_if_mngr, sync_wg_mngr);

        warp::serve(routes).run(addr).await
    }

    pub async fn serve_unix<I, W>(unix_stream: UnixListenerStream, if_mngr: I, wg_mngr: W)
    where
        I: InterfaceManager + Send + 'static,
        W: WireguardManager + Send + 'static,
    {
        let sync_if_mngr = Arc::new(Mutex::new(if_mngr));
        let sync_wg_mngr = Arc::new(Mutex::new(wg_mngr));

        let routes = get_routes(sync_if_mngr, sync_wg_mngr);

        warp::serve(routes).run_incoming(unix_stream).await
    }
}
