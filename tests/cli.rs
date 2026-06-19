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
