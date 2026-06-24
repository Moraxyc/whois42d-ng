use std::path::PathBuf;
use std::time::Duration;

use whois42d_ng::server::Options;

#[test]
fn parses_daemon_options() {
    let options = Options::parse_from([
        "whois42d-ng",
        "--address",
        "127.0.0.1",
        "--port",
        "4343",
        "--registry",
        "resources/fixtures/registry-3011",
        "--timeout",
        "5",
    ])
    .expect("options should parse");

    assert_eq!(options.address, "127.0.0.1");
    assert_eq!(options.port, 4343);
    assert_eq!(
        options.registry,
        PathBuf::from("resources/fixtures/registry-3011")
    );
    assert_eq!(options.timeout, Duration::from_secs(5));
}

#[test]
fn parses_rdap_options() {
    let options = Options::parse_from([
        "whois42d-ng",
        "--rdap-address",
        "::1",
        "--rdap-port",
        "8443",
        "--rdap-base-url",
        "https://rdap.example.dn42",
        "--rdap-path",
        "/",
    ])
    .expect("options should parse");

    assert_eq!(options.rdap_address, "::1");
    assert_eq!(options.rdap_port, 8443);
    assert_eq!(options.rdap_base_url, "https://rdap.example.dn42");
    assert_eq!(options.rdap_path, "/");
    assert_eq!(options.rdap_listen_addr(), "[::1]:8443");
}

#[test]
fn rdap_defaults_are_disabled() {
    let options = Options::default();

    assert_eq!(options.rdap_address, "");
    assert_eq!(options.rdap_port, 0);
    assert_eq!(options.rdap_base_url, "");
    assert_eq!(options.rdap_path, "/rdap");
}

#[test]
fn rejects_star_rdap_address() {
    let err = Options::parse_from(["whois42d-ng", "--rdap-address", "*"])
        .expect_err("star bind address should be rejected");

    assert!(err.contains("'*' is not supported"));
}

#[test]
fn rejects_relative_rdap_path() {
    let err = Options::parse_from(["whois42d-ng", "--rdap-path", "rdap"])
        .expect_err("relative rdap path should be rejected");

    assert!(err.contains("path must start with '/'"));
}

#[test]
fn validates_registry_data_path() {
    let options = Options::parse_from([
        "whois42d-ng",
        "--registry",
        "resources/fixtures/registry-3011",
    ])
    .expect("options should parse");

    assert_eq!(
        options
            .registry_data_path()
            .expect("data path should exist"),
        PathBuf::from("resources/fixtures/registry-3011/data")
    );
}
