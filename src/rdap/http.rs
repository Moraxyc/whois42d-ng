use std::net::IpAddr;

use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::get};
use ipnet::IpNet;

use crate::rdap::mapper;
use crate::rdap::model::RdapError;
use crate::registry::Registry;

#[derive(Debug, Clone)]
pub struct RdapState {
    pub registry: Registry,
    pub base_url: Option<String>,
    pub path: String,
}

pub fn routes(state: RdapState) -> Router {
    let path = state.path.clone();
    let router = Router::new()
        .route("/autnum/{asn}", get(handle_autnum))
        .route("/ip/{addr}", get(handle_ip))
        .route("/ip/{addr}/{prefix}", get(handle_ip_prefix))
        .route("/domain/{name}", get(handle_domain))
        .route("/entity/{handle}", get(handle_entity))
        .fallback(handle_not_found)
        .with_state(state);
    if path == "/" {
        router
    } else {
        Router::new().nest(&path, router)
    }
}

async fn handle_autnum(State(state): State<RdapState>, Path(asn): Path<String>) -> Response {
    let Some(name) = autnum_name(&asn) else {
        return error(StatusCode::BAD_REQUEST, "invalid autnum");
    };
    let registry = state.registry.clone();
    let result =
        tokio::task::spawn_blocking(move || registry.lookup_object("aut-num", &name)).await;
    let object = match lookup_one(result) {
        Ok(Some(object)) => object,
        Ok(None) => return error(StatusCode::NOT_FOUND, "object not found"),
        Err(err) => {
            log::warn!("rdap autnum lookup failed: {err}");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed");
        }
    };
    rdap_json(
        StatusCode::OK,
        mapper::autnum(&object, state.base_url.as_deref(), &state.path, &asn),
    )
}

async fn handle_domain(State(state): State<RdapState>, Path(name): Path<String>) -> Response {
    if unsafe_path(&name) {
        return error(StatusCode::BAD_REQUEST, "invalid domain");
    }
    let name = name.to_ascii_lowercase();
    let registry = state.registry.clone();
    let lookup_name = name.clone();
    let result =
        tokio::task::spawn_blocking(move || registry.lookup_object("dns", &lookup_name)).await;
    let object = match lookup_one(result) {
        Ok(Some(object)) => object,
        Ok(None) => return error(StatusCode::NOT_FOUND, "object not found"),
        Err(err) => {
            log::warn!("rdap domain lookup failed: {err}");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed");
        }
    };
    rdap_json(
        StatusCode::OK,
        mapper::domain(&object, state.base_url.as_deref(), &state.path, &name),
    )
}

async fn handle_entity(State(state): State<RdapState>, Path(handle): Path<String>) -> Response {
    if unsafe_path(&handle) {
        return error(StatusCode::BAD_REQUEST, "invalid entity");
    }
    let handle = handle.to_ascii_uppercase();
    let registry = state.registry.clone();
    let lookup_handle = handle.clone();
    let result = tokio::task::spawn_blocking(move || {
        if let Some(object) = registry.lookup_object("person", &lookup_handle)? {
            Ok(Some(object))
        } else {
            registry.lookup_object("mntner", &lookup_handle)
        }
    })
    .await;
    let object = match lookup_one(result) {
        Ok(Some(object)) => object,
        Ok(None) => return error(StatusCode::NOT_FOUND, "object not found"),
        Err(err) => {
            log::warn!("rdap entity lookup failed: {err}");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed");
        }
    };
    rdap_json(
        StatusCode::OK,
        mapper::entity(&object, state.base_url.as_deref(), &state.path, &handle),
    )
}

async fn handle_ip(State(state): State<RdapState>, Path(addr): Path<String>) -> Response {
    let Ok(addr_value) = addr.parse::<IpAddr>() else {
        return error(StatusCode::BAD_REQUEST, "invalid ip address");
    };
    let registry = state.registry.clone();
    let result = tokio::task::spawn_blocking(move || registry.lookup_ip(addr_value)).await;
    let objects = match lookup_many(result) {
        Ok(objects) if !objects.is_empty() => objects,
        Ok(_) => return error(StatusCode::NOT_FOUND, "object not found"),
        Err(err) => {
            log::warn!("rdap ip lookup failed: {err}");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed");
        }
    };
    rdap_json(
        StatusCode::OK,
        mapper::ip_network(&objects, state.base_url.as_deref(), &state.path, &addr)
            .expect("non-empty objects"),
    )
}

async fn handle_ip_prefix(
    State(state): State<RdapState>,
    Path((addr, prefix)): Path<(String, String)>,
) -> Response {
    let Ok(prefix_num) = prefix.parse::<u8>() else {
        return error(StatusCode::BAD_REQUEST, "invalid prefix");
    };
    let Ok(ip) = addr.parse::<IpAddr>() else {
        return error(StatusCode::BAD_REQUEST, "invalid ip address");
    };
    let Ok(network) = IpNet::new(ip, prefix_num) else {
        return error(StatusCode::BAD_REQUEST, "invalid prefix");
    };
    let object_type = match network {
        IpNet::V4(_) => "route",
        IpNet::V6(_) => "route6",
    };
    let object_name = format!("{}_{}", network.network(), network.prefix_len());
    let registry = state.registry.clone();
    let lookup_name = object_name.clone();
    let result =
        tokio::task::spawn_blocking(move || registry.lookup_object(object_type, &lookup_name))
            .await;
    let object = match lookup_one(result) {
        Ok(Some(object)) => object,
        Ok(None) => return error(StatusCode::NOT_FOUND, "object not found"),
        Err(err) => {
            log::warn!("rdap ip prefix lookup failed: {err}");
            return error(StatusCode::INTERNAL_SERVER_ERROR, "lookup failed");
        }
    };
    let query = format!("{}/{}", network.network(), network.prefix_len());
    rdap_json(
        StatusCode::OK,
        mapper::ip_network(&[object], state.base_url.as_deref(), &state.path, &query)
            .expect("one object"),
    )
}

async fn handle_not_found() -> Response {
    error(StatusCode::NOT_FOUND, "not found")
}

fn autnum_name(value: &str) -> Option<String> {
    let digits = value
        .strip_prefix("AS")
        .or_else(|| value.strip_prefix("as"))
        .unwrap_or(value);
    digits
        .chars()
        .all(|ch| ch.is_ascii_digit())
        .then(|| format!("AS{digits}"))
}

fn lookup_one<T>(
    result: Result<std::io::Result<Option<T>>, tokio::task::JoinError>,
) -> std::io::Result<Option<T>> {
    result.map_err(std::io::Error::other)?
}

fn lookup_many<T>(
    result: Result<std::io::Result<Vec<T>>, tokio::task::JoinError>,
) -> std::io::Result<Vec<T>> {
    result.map_err(std::io::Error::other)?
}

fn rdap_json(status: StatusCode, body: impl serde::Serialize) -> Response {
    (
        status,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/rdap+json"),
            ),
            (
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            ),
        ],
        Json(body),
    )
        .into_response()
}

fn error(status: StatusCode, title: &str) -> Response {
    rdap_json(
        status,
        RdapError {
            error_code: status.as_u16(),
            title: title.to_string(),
            description: vec![title.to_string()],
        },
    )
}

fn unsafe_path(value: &str) -> bool {
    value.contains('/') || value.contains("..")
}
