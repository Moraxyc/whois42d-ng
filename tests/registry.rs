use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use whois42d_ng::registry::Registry;

fn fixture_registry() -> Registry {
    Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"))
}

fn temp_registry_path(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "whois42d-ng-{label}-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn renders_existing_registry_object() {
    let response = fixture_registry()
        .handle_query("AS4242423011")
        .expect("query should render");

    assert!(response.contains("% This is the dn42 whois query service."));
    assert!(response.contains("% Information related to 'aut-num/AS4242423011':"));
    assert!(response.contains("aut-num:            AS4242423011"));
}

#[test]
fn returns_404_for_missing_registry_object() {
    let response = fixture_registry()
        .handle_query("AS4242423999")
        .expect("query should render");

    assert!(response.contains("% 404"));
}

#[test]
fn applies_type_filter() {
    let response = fixture_registry()
        .handle_query("-T person AS4242423011")
        .expect("query should render");

    assert!(response.contains("% 404"));
    assert!(!response.contains("aut-num:            AS4242423011"));
}

#[test]
fn matches_route_objects_containing_ip_addresses() {
    let response = fixture_registry()
        .handle_query("172.21.86.193")
        .expect("query should render");

    assert!(response.contains("route:              172.21.86.192/27"));
}

#[test]
fn matches_route_objects_for_cidr_queries() {
    let response = fixture_registry()
        .handle_query("172.21.86.192/27")
        .expect("query should render");

    assert!(response.contains("route:              172.21.86.192/27"));
}

#[test]
fn renders_unsupported_template_query_response() {
    let response = fixture_registry()
        .handle_query("-t person")
        .expect("query should render");

    assert!(response.contains("% template queries are unsupported for person"));
}

#[test]
fn renders_invalid_query_response() {
    let response = fixture_registry()
        .handle_query("-x nope")
        .expect("query should render");

    assert!(response.contains("% error: invalid query"));
}

#[test]
fn renders_existing_telephony_object() {
    let response = fixture_registry()
        .handle_query("+04243011")
        .expect("query should render");

    assert!(response.contains("% This is the dn42 whois query service."));
    assert!(response.contains("% Information related to 'telephony/+04243011':"));
    assert!(response.contains("telephony:          +04243011"));
    assert!(response.contains("nserver:            any.moraxyc.dn42"));
}

#[test]
fn refuses_path_traversal_in_whois_and_structured_lookups() {
    let data_path = temp_registry_path("registry-traversal");
    fs::create_dir_all(data_path.join("mntner")).expect("mntner directory should be created");
    let outside = data_path.join("OUTSIDE-MNT");
    fs::write(&outside, "mntner: OUTSIDE-MNT\n").expect("outside object should be created");
    let registry = Registry::new(data_path.clone());

    let response = registry
        .handle_query("../OUTSIDE-MNT")
        .expect("query should render");

    assert!(response.contains("% 404"));
    assert!(!response.contains("mntner: OUTSIDE-MNT"));

    for (object_type, object_name) in [
        ("mntner", "../OUTSIDE-MNT"),
        (".", "OUTSIDE-MNT"),
        (data_path.to_str().unwrap(), "OUTSIDE-MNT"),
        ("mntner", outside.to_str().unwrap()),
    ] {
        assert!(
            registry
                .lookup_object(object_type, object_name)
                .expect("lookup should not fail")
                .is_none(),
            "{object_type}/{object_name}"
        );
    }

    fs::remove_dir_all(data_path).expect("temporary registry should be removed");
}

#[test]
fn looks_up_structured_object() {
    let object = fixture_registry()
        .lookup_object("aut-num", "AS4242423011")
        .expect("lookup should not fail")
        .expect("object should exist");

    assert_eq!(object.object_type, "aut-num");
    assert_eq!(object.object_name, "AS4242423011");
    assert!(object.raw_text.contains("aut-num:            AS4242423011"));
    assert_eq!(object.rpsl.get("as-name"), Some("MORAXYC-AS"));
}

#[test]
fn structured_lookup_returns_read_errors() {
    let data_path = temp_registry_path("registry-read-error");
    let object_path = data_path.join("aut-num").join("AS4242423999");
    fs::create_dir_all(&object_path).expect("directory object path should be created");

    let err = Registry::new(data_path.clone())
        .lookup_object("aut-num", "AS4242423999")
        .expect_err("directory object path should return an I/O error");

    let _ = fs::remove_dir_all(data_path);

    assert_ne!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn looks_up_ip_objects_by_longest_prefix_first() {
    let data_path = temp_registry_path("registry-prefix-order");
    for (object_type, name, content) in [
        ("inetnum", "172.21.86.0_24", "inetnum: 172.21.86.0/24\n"),
        ("route", "172.21.86.192_27", "route: 172.21.86.192/27\n"),
        ("route", "172.21.86.192_28", "route: 172.21.86.192/28\n"),
        ("route", "172.21.86.224_28", "route: 172.21.86.224/28\n"),
    ] {
        fs::create_dir_all(data_path.join(object_type))
            .expect("object directory should be created");
        fs::write(data_path.join(object_type).join(name), content)
            .expect("network object should be created");
    }

    let objects = Registry::new(data_path.clone())
        .lookup_ip(IpAddr::V4(Ipv4Addr::new(172, 21, 86, 193)))
        .expect("lookup should not fail");
    fs::remove_dir_all(data_path).expect("temporary registry should be removed");

    assert_eq!(
        objects
            .iter()
            .map(|object| (object.object_type.as_str(), object.object_name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("route", "172.21.86.192_28"),
            ("route", "172.21.86.192_27"),
            ("inetnum", "172.21.86.0_24"),
        ]
    );
    assert_eq!(objects[0].rpsl.get("route"), Some("172.21.86.192/28"));
}
