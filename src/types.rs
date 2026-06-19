pub const ALL_TYPES: &[&str] = &[
    "aut-num",
    "dns",
    "person",
    "mntner",
    "schema",
    "organisation",
    "tinc-keyset",
    "tinc-key",
    "key-cert",
    "as-set",
    "route-set",
    "inetnum",
    "inet6num",
    "route",
    "route6",
    "as-block",
];

pub fn candidate_types(object: &str) -> Vec<&'static str> {
    let upper = object.to_ascii_uppercase();
    let lower = object.to_ascii_lowercase();
    let mut result = Vec::new();

    if upper.starts_with("AS")
        && !upper[2..].is_empty()
        && upper[2..].chars().all(|c| c.is_ascii_digit())
    {
        result.push("aut-num");
    }
    if lower.ends_with(".dn42") {
        result.push("dns");
    }
    if upper.ends_with("-DN42") || upper.ends_with("-NEONETWORK") {
        result.push("person");
    }
    if upper.ends_with("-MNT") {
        result.push("mntner");
    }
    if upper.ends_with("-SCHEMA") {
        result.push("schema");
    }
    if upper.starts_with("ORG-") {
        result.push("organisation");
    }
    if upper.starts_with("SET-") && upper.ends_with("-TINC") {
        result.push("tinc-keyset");
    } else if upper.ends_with("-TINC") {
        result.push("tinc-key");
    }
    if upper.starts_with("PGPKEY-") {
        result.push("key-cert");
    }
    if upper.starts_with("AS") {
        result.push("as-set");
    }
    if upper.starts_with("RS-") {
        result.push("route-set");
    }
    if is_as_block(&upper) {
        result.push("as-block");
    }

    result
}

fn is_as_block(object: &str) -> bool {
    let Some((left, right)) = object.split_once('_') else {
        return false;
    };
    !left.is_empty()
        && !right.is_empty()
        && left.chars().all(|c| c.is_ascii_digit())
        && right.chars().all(|c| c.is_ascii_digit())
}
