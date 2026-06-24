use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use whois42d_ng::registry::Registry;

fn fixture_registry() -> Registry {
    Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"))
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
fn refuses_path_traversal_queries() {
    let outside = PathBuf::from("resources/fixtures/registry-3011/secret");
    fs::write(&outside, "secret").expect("test secret should be writable");

    let response = fixture_registry()
        .handle_query("../secret")
        .expect("query should render");

    assert!(response.contains("% 404"));
    assert!(!response.contains("secret"));

    fs::remove_file(outside).expect("test secret should be removed");
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
fn structured_lookup_refuses_path_traversal() {
    let object = fixture_registry()
        .lookup_object("aut-num", "../secret")
        .expect("lookup should not fail");

    assert!(object.is_none());
}

#[test]
fn looks_up_ip_objects_by_longest_prefix_first() {
    let objects = fixture_registry()
        .lookup_ip(IpAddr::V4(Ipv4Addr::new(172, 21, 86, 193)))
        .expect("lookup should not fail");

    assert_eq!(objects[0].object_type, "route");
    assert_eq!(objects[0].object_name, "172.21.86.192_27");
    assert_eq!(objects[0].rpsl.get("route"), Some("172.21.86.192/27"));
}
