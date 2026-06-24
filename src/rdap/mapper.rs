use ipnet::IpNet;

use crate::rdap::model::{EntityRef, Link, RdapObject, Remark};
use crate::registry::ObjectRef;

pub fn autnum(object: &ObjectRef, base_url: Option<&str>, query: &str) -> RdapObject {
    let handle = object.object_name.to_ascii_uppercase();
    let mut response = base_object("autnum", handle.clone(), base_url, "autnum", query, &handle);
    if let Ok(value) = handle.trim_start_matches("AS").parse::<u64>() {
        response.start_autnum = Some(value);
        response.end_autnum = Some(value);
    }
    response.name = object.rpsl.get("as-name").map(str::to_string);
    response.entities = entity_refs(object);
    response.remarks = remarks(
        object,
        &[
            "aut-num", "as-name", "admin-c", "tech-c", "zone-c", "source",
        ],
    );
    response
}

pub fn domain(object: &ObjectRef, base_url: Option<&str>, query: &str) -> RdapObject {
    let handle = object.object_name.to_ascii_lowercase();
    let mut response = base_object("domain", handle.clone(), base_url, "domain", query, &handle);
    response.ldh_name = Some(handle.clone());
    response.entities = entity_refs(object);
    response.remarks = remarks(object, &["domain", "admin-c", "tech-c", "zone-c", "source"]);
    response
}

pub fn entity(object: &ObjectRef, base_url: Option<&str>, query: &str) -> RdapObject {
    let handle = object.object_name.to_ascii_uppercase();
    let mut response = base_object("entity", handle.clone(), base_url, "entity", query, &handle);
    let name = object
        .rpsl
        .get("person")
        .or_else(|| object.rpsl.get("mntner"))
        .unwrap_or(&handle);
    response.vcard_array = Some(serde_json::json!(["vcard", [["fn", {}, "text", name]]]));
    response.remarks = remarks(object, &["person", "nic-hdl", "mntner", "source"]);
    response
}

pub fn ip_network(
    objects: &[ObjectRef],
    base_url: Option<&str>,
    query: &str,
) -> Option<RdapObject> {
    let object = objects.first()?;
    let mut response = base_object(
        "ip network",
        object.object_name.clone(),
        base_url,
        "ip",
        query,
        query,
    );
    if let Some((start, end, version)) = network_range(&object.object_name) {
        response.start_address = Some(start);
        response.end_address = Some(end);
        response.ip_version = Some(version);
    }
    response.entities = entity_refs(object);
    response.remarks = objects
        .iter()
        .flat_map(|object| {
            let primary = object
                .rpsl
                .get(&object.object_type)
                .unwrap_or(&object.object_name)
                .to_string();
            let mut remarks = vec![Remark {
                title: object.object_type.clone(),
                description: vec![primary],
            }];
            remarks.extend(remarks_for_known_text(object));
            remarks
        })
        .collect();
    Some(response)
}

fn base_object(
    class_name: &str,
    handle: String,
    base_url: Option<&str>,
    route: &str,
    value_path: &str,
    href_path: &str,
) -> RdapObject {
    let value_path = value_path.trim_start_matches('/');
    let href_path = href_path.trim_start_matches('/');
    RdapObject {
        rdap_conformance: vec!["rdap_level_0".to_string()],
        object_class_name: class_name.to_string(),
        handle,
        ldh_name: None,
        name: None,
        links: base_url
            .map(|base_url| Link {
                value: format!(
                    "{}/rdap/{route}/{value_path}",
                    base_url.trim_end_matches('/')
                ),
                rel: "self".to_string(),
                href: format!(
                    "{}/rdap/{route}/{href_path}",
                    base_url.trim_end_matches('/')
                ),
                media_type: "application/rdap+json".to_string(),
            })
            .into_iter()
            .collect(),
        entities: Vec::new(),
        start_autnum: None,
        end_autnum: None,
        start_address: None,
        end_address: None,
        ip_version: None,
        remarks: Vec::new(),
        notices: vec![Remark {
            title: "Service Notice".to_string(),
            description: vec!["This RDAP service provides DN42 registry data.".to_string()],
        }],
        status: vec!["active".to_string()],
        vcard_array: None,
    }
}

fn network_range(name: &str) -> Option<(String, String, String)> {
    match name.replace('_', "/").parse::<IpNet>().ok()? {
        IpNet::V4(net) => Some((
            net.network().to_string(),
            net.broadcast().to_string(),
            "v4".to_string(),
        )),
        IpNet::V6(net) => Some((
            net.network().to_string(),
            net.broadcast().to_string(),
            "v6".to_string(),
        )),
    }
}

fn entity_refs(object: &ObjectRef) -> Vec<EntityRef> {
    [
        ("admin-c", "administrative"),
        ("tech-c", "technical"),
        ("zone-c", "zone"),
    ]
    .into_iter()
    .flat_map(|(key, role)| {
        object
            .rpsl
            .get_all(key)
            .into_iter()
            .map(move |handle| EntityRef {
                object_class_name: "entity".to_string(),
                handle: handle.to_ascii_uppercase(),
                roles: vec![role.to_string()],
            })
    })
    .collect()
}

fn remarks(object: &ObjectRef, known: &[&str]) -> Vec<Remark> {
    let mut values = remarks_for_known_text(object);
    values.extend(
        object
            .rpsl
            .fields
            .iter()
            .filter(|(key, _)| !known.iter().any(|known| key.eq_ignore_ascii_case(known)))
            .map(|(key, value)| Remark {
                title: key.clone(),
                description: vec![value.clone()],
            }),
    );
    values
}

fn remarks_for_known_text(object: &ObjectRef) -> Vec<Remark> {
    object
        .rpsl
        .fields
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("descr") || key.eq_ignore_ascii_case("remarks"))
        .map(|(key, value)| Remark {
            title: key.clone(),
            description: vec![value.clone()],
        })
        .collect()
}
