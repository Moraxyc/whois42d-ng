# Debian / Ubuntu packages

Packages contain a statically linked Linux binary for `amd64` or `arm64`,
with systemd socket activation support and no Nix runtime dependency.
Installation does not enable or start the units.
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

With Nix installed on Linux:

```sh
nix build .#whois42d-ng-deb
sudo apt-get install ./result/whois42d-ng_*.deb
```

The `.deb` is assembled directly with `dpkg-deb` from the static Nix build.
To build just the release binary:

```sh
nix build .#whois42d-ng-static
./result/bin/whois42d-ng --help
```

Releases provide binaries for `x86_64-linux`, `aarch64-linux`, and
`aarch64-darwin`, plus `.deb` packages for the two Linux architectures.
Linux binaries are fully static; the macOS binary still uses system libraries.

## Publish

Update the version in `Cargo.toml` and `Cargo.lock`, commit the changes, then
push a matching `vX.Y.Z` tag. The Release workflow checks the version, builds
the release artifacts, and generates release notes from Conventional Commits
using `git-cliff` and `cliff.toml`. Release notes are published on
[GitHub Releases](https://github.com/Moraxyc/whois42d-ng/releases).

Publishing uses the workflow’s `GITHUB_TOKEN`; no GitHub App secrets are needed.
The APT repository picks up releases on its next scheduled refresh.
