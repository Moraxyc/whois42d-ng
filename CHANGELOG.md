# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [1.0.1](https://github.com/Moraxyc/whois42d-ng/compare/v1.0.0...v1.0.1) - 2026-06-19


### 🐛 Bug Fixes


- Use explicit 127.0.0.1 for empty bind address



### 💬 Other


- Switch to musl build on Alpine



### 📚 Documentation


- Update README to reference whois42d-ng service/socket files



### 👷 CI


- Update

- Add cargo-semver-checks

- Use release-pr

- Add changelog template


## [1.0.0] - 2026-06-19

### 🚀 Features

- Initialize rust port with registry core
- Add tcp server and cli options
- Add daemon runtime with socket activation
- Port latest upstream changes
- Use ipnet crate
- Use systemd crate
- Sd notify
- *(protocol)* Add bounded query log text sanitizer
- *(main)* Initialize env_logger with info default
- Add structured lifecycle, warning, and debug logging
- Support telephony

### 🐛 Bug Fixes

- *(server)* Bracket IPv6 addresses in listen addr display

### 💼 Other

- Add nix flake and packaging

### 📚 Documentation

- Update readme for rust port

### 🧪 Testing

- Add integration tests and registry fixtures

### ⚙️ Miscellaneous Tasks

- Remove legacy go implementation
- Remove travis
- Add build and renovate
- Limit flake checks to source changes
- Harden systemd units
- Add release-plz
- Use release-plz from nixpkgs
- Fix release-plz
- Add changelog
- Set git_only to release-plz.toml
- Exclude fixtures
