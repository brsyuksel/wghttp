use std::net::SocketAddr;

use argh::FromArgs;

use if_mng_libc::*;
use wg_mng_cbind::*;

mod api;

const WGHTTP_SOCKET: &str = "/var/run/wghttp.socket";

#[derive(FromArgs)]
/// A web server manages wireguard devices.
struct Args {
    /// unix socket path, default: /var/run/wghttp.socket
    #[argh(option)]
    unix: Option<String>,

    /// tcp address to listen, e.g., 0.0.0.0:9204
    #[argh(option)]
    tcp: Option<String>,
}

#[cfg(unix)]
#[tokio::main]
async fn main() {
    let args: Args = argh::from_env();

    let if_mngr = InterfaceManagerLibC;
    let wg_mngr = WireguardManagerCBind;

    if let Some(addr) = args.tcp {
        let sock_addr = addr.parse::<SocketAddr>().unwrap();
        api::handlers::server::serve_tcp(sock_addr, if_mngr, wg_mngr).await;
        return
    }

    use tokio::net::UnixListener;
    use tokio_stream::wrappers::UnixListenerStream;

    let unix_path = args.unix.unwrap_or(WGHTTP_SOCKET.to_owned());
    let listener = UnixListener::bind(unix_path).unwrap();
    let stream = UnixListenerStream::new(listener);
    api::handlers::server::serve_unix(stream, if_mngr, wg_mngr).await;
}

#[cfg(not(unix))]
#[tokio::main]
async fn main() {
    panic!("wghttp only supports unix-like envs")
}
