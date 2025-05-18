# wghttp

**wghttp** is a zero-log, lightweight, and opinionated HTTP server for managing WireGuard devices and peers.

It’s particularly useful when you need to control your WireGuard setup remotely.  
It also simplifies the process of adding devices or peers — allowing you to do it with a single HTTP call.  
`wghttp` saves time, especially when adding new peers to your VPN server without needing to SSH in.


## Features

- RESTful HTTP API for managing WireGuard interfaces and peers
- Runs on **Unix domain socket** by default (`/var/run/wghttp.sock`)
- Can be configured to run over TCP (`--tcp ip:port`)
- Swagger UI available at `/swagger-ui/` for API exploration

## Usage

```bash
cargo build --release
./target/release/wghttp --help
```

### Launch Examples

#### Using Unix domain socket (recommended):

```bash
sudo ./wghttp
```

#### Using TCP:

```bash
sudo ./wghttp --tcp 127.0.0.1:8080
```

> **Note:** Unix domain socket is preferred since it delegates authentication to the system. Users cannot send curl requests without `sudo`.

### Permissions

`wghttp` interacts with networking interfaces and requires elevated privileges.

- The application **must be run with `sudo`** unless the `CAP_NET_ADMIN` capability is explicitly granted.
- **Granting `CAP_NET_ADMIN` is not recommended** due to potential security implications.

```bash
sudo setcap cap_net_admin+ep ./target/release/wghttp
```

### Authentication & TLS (via Caddy)

To secure `wghttp` behind HTTPS and add basic authentication, you can use [Caddy](https://caddyserver.com/) as a reverse proxy.

Below is an example `Caddyfile` that:

- Listens on port 443 with HTTPS
- Applies HTTP Basic Auth
- Uses a self-signed TLS certificate
- Proxies traffic to `wghttp` over a Unix domain socket

```caddyfile
https://[::]:443, https://:443 {
	reverse_proxy unix//var/run/wghttp.sock

	basic_auth {
		your_username $2a$14$zBNAL8oUW/m3vpTIjm2ts.M64u2JKRvZJkd2bw/kKDSV3tniHWPuW
	}

	tls /path/to/self-signed.crt /path/to/self-signed.key
}
```

> You can generate a bcrypt hash for your password using `caddy hash-password` interactive command.

## Swagger UI

To explore the API and send test requests:

```
http://localhost/swagger-ui/
```

> The interface uses OpenAPI 3.0 specification.

## Tests

Some tests require access to system commands:

### Dependencies

- `ip` command (via `iproute2` package)
- `wg` command (via `wireguard-tools` package)

### Installation Examples by Distribution

| Distribution | Install Command |
|--------------|------------------|
| Arch Linux   | `sudo pacman -S iproute2 wireguard-tools` |
| Ubuntu/Debian | `sudo apt install iproute2 wireguard-tools` |
| Fedora       | `sudo dnf install iproute wireguard-tools` |

### Running Tests

#### HTTP route tests:

```bash
cargo test -p wghttp
```

#### `netdev` tests (requires root):

```bash
sudo -E $(which cargo) test -p netdev
```

> `netdev` tests depend on `ip`

#### `wgshim` tests (should run single-threaded):

```bash
sudo -E $(which cargo) test -p wgshim -- --test-threads 1
```

> `wgshim` tests depend on both `ip` and `wg` commands.

## License

This project includes components from the WireGuard embedding library (wireguard.c and wireguard.h), which are licensed under the LGPL-2.1+ license. All other parts of this project are licensed under the MIT License.

See `LICENSE` and individual source files for more information.
