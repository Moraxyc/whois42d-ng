use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use ipnet::IpNet;
use serde::Serialize;

use crate::registry::Registry;

/// IANA-format RDAP bootstrap registry file (RFC 7484 / RFC 9224).
///
/// `services` pairs each list of resource keys with the URLs of the
/// authoritative RDAP servers covering them. This server advertises a single
/// authoritative URL, so every file has one service tuple.
#[derive(Debug, Serialize)]
pub struct BootstrapFile {
    version: String,
    publication: String,
    services: Vec<(Vec<String>, Vec<String>)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Asn,
    Dns,
    Ipv4,
    Ipv6,
}

impl Kind {
    pub fn from_file(name: &str) -> Option<Self> {
        match name {
            "asn.json" => Some(Self::Asn),
            "dns.json" => Some(Self::Dns),
            "ipv4.json" => Some(Self::Ipv4),
            "ipv6.json" => Some(Self::Ipv6),
            _ => None,
        }
    }
}

pub fn build(
    registry: &Registry,
    kind: Kind,
    base_url: Option<&str>,
    path: &str,
) -> io::Result<BootstrapFile> {
    let url = service_base_url(base_url, path);
    let entries = match kind {
        Kind::Asn => autnum_entries(registry)?,
        Kind::Dns => dns_entries(registry)?,
        Kind::Ipv4 => ip_entries(registry, &["inetnum", "route"], false)?,
        Kind::Ipv6 => ip_entries(registry, &["inet6num", "route6"], true)?,
    };
    Ok(BootstrapFile {
        version: "1.0".to_string(),
        publication: rfc3339_utc(SystemTime::now()),
        services: vec![(entries, vec![url])],
    })
}

fn autnum_entries(registry: &Registry) -> io::Result<Vec<String>> {
    let mut entries: Vec<String> = registry
        .list_object_names("aut-num")?
        .into_iter()
        .filter_map(|name| {
            let rest = name
                .strip_prefix("AS")
                .or_else(|| name.strip_prefix("as"))?;
            (!rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())).then(|| rest.to_string())
        })
        .collect();
    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn dns_entries(registry: &Registry) -> io::Result<Vec<String>> {
    let mut entries: Vec<String> = registry
        .list_object_names("dns")?
        .into_iter()
        .filter_map(|name| name.split('.').next_back().map(str::to_string))
        .filter(|tld| !tld.is_empty())
        .collect();
    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn ip_entries(registry: &Registry, types: &[&str], ipv6: bool) -> io::Result<Vec<String>> {
    let mut entries = Vec::new();
    for object_type in types {
        for name in registry.list_object_names(object_type)? {
            let cidr = name.replace('_', "/");
            let matches = match cidr.parse::<IpNet>() {
                Ok(IpNet::V4(_)) => !ipv6,
                Ok(IpNet::V6(_)) => ipv6,
                Err(_) => false,
            };
            if matches {
                entries.push(cidr);
            }
        }
    }
    entries.sort();
    entries.dedup();
    Ok(entries)
}

fn service_base_url(base_url: Option<&str>, path: &str) -> String {
    let base = base_url.unwrap_or("").trim_end_matches('/');
    let prefix = if path == "/" {
        ""
    } else {
        path.trim_end_matches('/')
    };
    format!("{base}{prefix}/")
}

fn rfc3339_utc(now: SystemTime) -> String {
    let secs = now
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let hour = (rem / 3600) as u8;
    let minute = ((rem % 3600) / 60) as u8;
    let second = (rem % 60) as u8;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = if mp < 10 {
        (mp + 3) as u8
    } else {
        (mp - 9) as u8
    };
    let year = if month <= 2 { y + 1 } else { y };

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
