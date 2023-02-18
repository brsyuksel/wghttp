# wghttp

a web server helps you managing your wireguard devices and peers.

## how to use

run `wghttp` command to listen unix socket `/var/run/wghttp.socket`, you can also specify path by using `--unix PATH` option.
if you want to make it listening tcp use `--tcp SOCK_ADDR` option, e.g, `--tcp 0.0.0.0:9204`

see `wghttp --help` for details.

> NOTE THAT `wghttp` needs root permissions since it manages devices, so I strongly suggest you to run it listening unix socket. using unix socket can leverage user permissions to your operating system.

### api

visit `/api-doc.json`, you will have openapi 3 specs, use a client supports openapi import like Postman.

## tests

tests depends `wg` and `ip` commands, so make sure you install relevant packages in your env before running tests.

## copyright

"WireGuard" and the "WireGuard" logo are registered trademarks of Jason A. Donenfeld.
