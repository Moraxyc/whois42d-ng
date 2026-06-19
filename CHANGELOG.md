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
