# wghttp

**wghttp** is a lightweight, opinionated HTTP server designed to manage WireGuard devices and peers.
It is mostly useful when you need to control your devices or peers
remotely. In addition to that, it also reduces the steps required to add
a device or peer by allowing you to do it with a single http call. `wghttp`
saves time, especially when you want to add a new peer to your vpn server
without having to dive into ssh.

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
