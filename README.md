# whois42d-ng

[![Nix Flake Check](https://github.com/Moraxyc/whois42d-ng/actions/workflows/nix-flake-check.yml/badge.svg)](https://github.com/Moraxyc/whois42d-ng/actions/workflows/nix-flake-check.yml)

WHOIS server for the dn42 registry, written in Rust.

Originally based on `whoisd` from the dn42 monotone registry by welterde.

## Installation

Build with Cargo:

    $ cargo build --release

## Usage

    $ target/release/whois42d-ng --registry /path/to/registry

The daemon serves the WHOIS protocol on TCP port 43 by default. See
[Options](#options) for binding, RDAP, and socket-activation behavior.

## Options

| Option | Default | Description |
| --- | --- | --- |
| `--address` | `""` (loopback) | Bind address. Use `0.0.0.0` or `::` for all interfaces. |
| `--port` | `43` | WHOIS TCP port. |
| `--registry` | `.` | Path to the registry root (its `data/` directory is served). |
| `--timeout` | `10` | Seconds a socket-activated instance stays idle before exiting. |
| `--rdap-address` | `""` (loopback) | RDAP HTTP bind address. |
| `--rdap-port` | `0` | RDAP HTTP port; `0` disables the RDAP listener. |
| `--rdap-path` | `/rdap` | RDAP URL path prefix. |
| `--rdap-base-url` | `""` | Base URL for RDAP self-links (e.g. `https://rdap.example.dn42`). |

### Shell completions

Generate completion scripts for your shell:

    $ whois42d-ng completions bash > whois42d-ng.bash

## Run without root

Binding port 43 normally requires root. Use one of the following to run
whois42d-ng unprivileged.

1. Grant the capability on the binary:

        $ setcap 'cap_net_bind_service=+ep' ./whois42d-ng

2. Use a supervisor with socket activation, for example systemd:

        $ cp whois42d-ng.service whois42d-ng.socket /etc/systemd/system
        $ install -D -m755 target/release/whois42d-ng /usr/local/bin/whois42d-ng

Edit `whois42d-ng.service` to point `--registry` at your registry path, then
enable the socket:

    $ systemctl enable --now whois42d-ng.socket

**NOTE**: Start the socket, not the service. Under socket activation systemd
creates the listening socket and hands it to the service; starting
`whois42d-ng.service` directly makes it bind the port itself, which the
hardened unit is not designed for.

## Supported Queries

- mntner: `$ whois -h <server> HAX404-MNT`
- person: `$ whois -h <server> HAX404-DN42`
- aut-num: `$ whois -h <server> AS4242420429`
- dns: `$ whois -h <server> hax404.dn42`
- inetnum: `$ whois -h <server> 172.23.136.0/23` or `$ whois -h <server> 172.23.136.1`
- inet6num: `$ whois -h <server> fd58:eb75:347d::/48`
- route: `$ whois -h <server> 172.23.136.0/23`
- route6: `$ whois -h <server> fdec:1:1:dead::/64`
- schema: `$ whois -h <server> PERSON-SCHEMA`
- organisation: `$ whois -h <server> ORG-C3D2`
- tinc-keyset: `$ whois -h <server> SET-1-DN42-TINC`
- tinc-key: `$ whois -h <server> <node>-TINC`
- key-cert: `$ whois -h <server> PGPKEY-12345678`
- as-set: `$ whois -h <server> AS-FREIFUNK`
- as-block: `$ whois -h <server> 4242420000_4242423999`
- route-set: `$ whois -h <server> RS-DN42-NATIVE`
- telephony: `$ whois -h <server> +04243011`
- version: `$ whois -h <server> -q version`
- sources: `$ whois -h <server> -q sources`
- types: `$ whois -h <server> -q types`
- type filtering: `$ whois -h <server> -T aut-num,person Mic92-DN42 AS4242420092`

Template queries using `-t` return an unsupported response.

IP and CIDR queries return all matching `inetnum`/`route` (IPv4) or
`inet6num`/`route6` (IPv6) objects, most specific first.

## RDAP HTTP Interface

The daemon can additionally serve an RDAP-over-HTTP interface. Enable it with
`--rdap-port` and, optionally, `--rdap-address`, `--rdap-path`, and
`--rdap-base-url`:

    $ whois42d-ng --registry /path/to/registry --rdap-port 1080

Supported lookups under the configured `--rdap-path` (default `/rdap`):

- autnum: `GET /rdap/autnum/AS4242423011`
- ip: `GET /rdap/ip/172.21.86.193` or `GET /rdap/ip/172.21.86.192/27`
- domain: `GET /rdap/domain/moraxyc.dn42`
- entity: `GET /rdap/entity/MORAXYC-DN42`

IANA-format RDAP bootstrap registry files (RFC 7484) are published under
`<rdap-path>/bootstrap/` so RDAP clients can discover this server as the
authoritative source for the registry's resources. Each file lists the
autonomous system numbers (`asn.json`), top-level domains (`dns.json`), IPv4
prefixes (`ipv4.json`), or IPv6 prefixes (`ipv6.json`) found in the registry,
all pointing at the configured `--rdap-base-url`:

    $ curl http://localhost:1080/rdap/bootstrap/asn.json
    $ curl http://localhost:1080/rdap/bootstrap/dns.json
    $ curl http://localhost:1080/rdap/bootstrap/ipv4.json
    $ curl http://localhost:1080/rdap/bootstrap/ipv6.json

Responses use `application/rdap+json` and are CORS-enabled
(`Access-Control-Allow-Origin: *`). A liveness probe is exposed at a stable
path outside the RDAP prefix:

    $ curl http://localhost:1080/healthz
    ok

## Build Docker Image

Build a local Docker image:

```
$ docker build -t whois42d-ng .
```

Run it with a local registry checkout mounted at `/registry`:

```
$ docker run -v path/to/registry:/registry -p 43:4343 --rm whois42d-ng
```

The image listens on port `4343`; map it to the host port you want to expose.
