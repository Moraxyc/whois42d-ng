# Debian / Ubuntu packages

Packages are built for Ubuntu 24.04 on `amd64` and `arm64`.
APT installation is available after this project is enabled in the central repository.

## Install

```sh
sudo install -d -m 0755 /etc/apt/keyrings
curl -fsSL https://pkgs.moraxyc.com/deb/keys/moraxyc-archive-keyring.gpg \
  | sudo tee /etc/apt/keyrings/moraxyc-archive-keyring.gpg > /dev/null
printf '%s\n' 'deb [signed-by=/etc/apt/keyrings/moraxyc-archive-keyring.gpg] https://pkgs.moraxyc.com/deb stable main' \
  | sudo tee /etc/apt/sources.list.d/moraxyc.list
sudo apt-get update
sudo apt-get install whois42d-ng
```

Place your dn42 registry checkout at `/var/lib/whois42d/registry`, then start WHOIS:

```sh
sudo systemctl enable --now whois42d-ng.socket
```

To use a different registry path, override `ExecStart` with
`sudo systemctl edit whois42d-ng.service`.
For RDAP, also enable `whois42d-ng-rdap.socket`.

## Build a package

With Rust installed on Ubuntu 24.04:

```sh
sudo apt-get install debhelper dpkg-dev pkg-config libsystemd-dev jq
bash scripts/build-deb.sh
```

The package is written to `dist/`. To install it locally:

```sh
sudo apt-get install ./dist/whois42d-ng_*.deb
```

## Publish

Update the version in `Cargo.toml` and `Cargo.lock`, commit the changes, then
push a matching `vX.Y.Z` tag. The Release workflow checks the version, builds
the release artifacts, and generates release notes from Conventional Commits
using `git-cliff` and `cliff.toml`. `CHANGELOG.md` is retained as historical
release notes; new releases do not require editing it.

Publishing uses the workflow’s `GITHUB_TOKEN`; no GitHub App secrets are needed.
The APT repository picks up releases on its next scheduled refresh.
