# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
## [2.0.2](https://github.com/Moraxyc/whois42d-ng/compare/v2.0.1...v2.0.2) - 2026-07-03


### 🐛 Bug Fixes


- Sd whois listender



### ⚙️ Miscellaneous Tasks


- Release v2.0.1


## [2.0.1](https://github.com/Moraxyc/whois42d-ng/compare/v2.0.0...v2.0.1) - 2026-07-03


### 🐛 Bug Fixes


- Sd whois listender



### 💬 Other


- Add rdap socket file


## [2.0.0](https://github.com/Moraxyc/whois42d-ng/compare/v1.1.0...v2.0.0) - 2026-07-03


### 🚀 Features


- Add CORS and self-links to RDAP entities

- Allow set --rdap-path

- Add RDAP HTTP JSON query interface



### 🐛 Bug Fixes


- Normalize RDAP IP prefix lookups

- Avoid duplicate RDAP text remarks

- Add RDAP domain ldhName

- Add RDAP IP network range fields

- Add RDAP autnum range fields

- Add RDAP service notice

- Add active status to RDAP objects

- Distinguish RDAP link value and href

- Include RDAP entity class on references



### 💬 Other


- Allow darwin build



### ⚡ Performance


- Improve RDAP error handling



### ♻️ Refactor


- Run whois server on tokio



### 👷 CI


- Enable nix lockfile maintaince


## [1.1.0](https://github.com/Moraxyc/whois42d-ng/compare/v1.0.1...v1.1.0) - 2026-06-19


### 🚀 Features


- Fall back to longest registered telephony prefix on exact miss



### 🐛 Bug Fixes


- Reject star bind address



### 👷 CI


- Create tag before pr


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
