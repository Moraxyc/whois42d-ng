use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct RdapError {
    #[serde(rename = "errorCode")]
    pub error_code: u16,
    pub title: String,
    pub description: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Link {
    pub value: String,
    pub rel: String,
    pub href: String,
    #[serde(rename = "type")]
    pub media_type: String,
}

#[derive(Debug, Serialize)]
pub struct Remark {
    pub title: String,
    pub description: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct EntityRef {
    #[serde(rename = "objectClassName")]
    pub object_class_name: String,
    pub handle: String,
    pub roles: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[derive(Debug, Serialize)]
pub struct RdapObject {
    #[serde(rename = "rdapConformance")]
    pub rdap_conformance: Vec<String>,
    #[serde(rename = "objectClassName")]
    pub object_class_name: String,
    pub handle: String,
    #[serde(rename = "ldhName", skip_serializing_if = "Option::is_none")]
    pub ldh_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub entities: Vec<EntityRef>,
    #[serde(rename = "startAutnum", skip_serializing_if = "Option::is_none")]
    pub start_autnum: Option<u64>,
    #[serde(rename = "endAutnum", skip_serializing_if = "Option::is_none")]
    pub end_autnum: Option<u64>,
    #[serde(rename = "startAddress", skip_serializing_if = "Option::is_none")]
    pub start_address: Option<String>,
    #[serde(rename = "endAddress", skip_serializing_if = "Option::is_none")]
    pub end_address: Option<String>,
    #[serde(rename = "ipVersion", skip_serializing_if = "Option::is_none")]
    pub ip_version: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub remarks: Vec<Remark>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<Remark>,
    pub status: Vec<String>,
    #[serde(rename = "vcardArray", skip_serializing_if = "Option::is_none")]
    pub vcard_array: Option<Value>,
}
