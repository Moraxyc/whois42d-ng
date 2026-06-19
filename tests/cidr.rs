use std::net::IpAddr;
use std::str::FromStr;

use whois42d_ng::cidr::RegistryCidr;

#[test]
fn parses_registry_cidr_file_names() {
    let cidr = RegistryCidr::parse_file_name("172.21.86.192_27").expect("cidr should parse");

    assert!(cidr.contains(IpAddr::from_str("172.21.86.193").unwrap()));
    assert!(!cidr.contains(IpAddr::from_str("172.21.86.254").unwrap()));
    assert_eq!(cidr.file_name(), "172.21.86.192_27");
}
