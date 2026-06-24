use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use std::path::PathBuf;
use tower::ServiceExt;
use whois42d_ng::rdap::http::{RdapState, routes};
use whois42d_ng::registry::Registry;

fn app() -> axum::Router {
    routes(RdapState {
        registry: Registry::new(PathBuf::from("resources/fixtures/registry-3011/data")),
        base_url: Some("https://rdap.example.dn42".to_string()),
    })
}

async fn get(path: &str) -> (StatusCode, String, Value) {
    let response = app()
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .expect("content type should exist")
        .to_str()
        .expect("content type should be ascii")
        .to_string();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json = serde_json::from_slice(&body).expect("body should be json");
    (status, content_type, json)
}

#[tokio::test]
async fn serves_autnum_with_and_without_as_prefix() {
    let (_, _, bare) = get("/rdap/autnum/4242423011").await;
    let (status, content_type, prefixed) = get("/rdap/autnum/AS4242423011").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/rdap+json");
    assert_eq!(bare["handle"], "AS4242423011");
    assert_eq!(prefixed["objectClassName"], "autnum");
    assert_eq!(prefixed["name"], "MORAXYC-AS");
    assert_eq!(prefixed["entities"][0]["objectClassName"], "entity");
    assert_eq!(
        prefixed["links"][0]["href"],
        "https://rdap.example.dn42/rdap/autnum/AS4242423011"
    );
}

#[tokio::test]
async fn serves_domain_after_lowercasing() {
    let (status, _, json) = get("/rdap/domain/MORAXYC.DN42").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "domain");
    assert_eq!(json["handle"], "moraxyc.dn42");
}

#[tokio::test]
async fn serves_person_entity() {
    let (status, _, json) = get("/rdap/entity/MORAXYC-DN42").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "entity");
    assert_eq!(json["handle"], "MORAXYC-DN42");
    assert_eq!(json["vcardArray"][1][0][3], "Moraxyc");
}

#[tokio::test]
async fn serves_ip_network_with_route_remark() {
    let (status, _, json) = get("/rdap/ip/172.21.86.193").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "ip network");
    assert_eq!(json["handle"], "172.21.86.192_27");
    assert_eq!(json["remarks"][0]["title"], "route");
}

#[tokio::test]
async fn invalid_autnum_returns_rdap_error() {
    let (status, content_type, json) = get("/rdap/autnum/not-an-asn").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(content_type, "application/rdap+json");
    assert_eq!(json["errorCode"], 400);
}

#[tokio::test]
async fn missing_object_returns_rdap_error() {
    let (status, _, json) = get("/rdap/autnum/4242423999").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["errorCode"], 404);
}
