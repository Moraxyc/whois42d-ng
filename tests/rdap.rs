use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tower::ServiceExt;
use whois42d_ng::rdap::http::{RdapState, routes};
use whois42d_ng::registry::Registry;

fn app() -> axum::Router {
    app_with_path("/rdap")
}

fn app_with_path(path: &str) -> axum::Router {
    routes(RdapState {
        registry: Registry::new(PathBuf::from("resources/fixtures/registry-3011/data")),
        base_url: Some("https://rdap.example.dn42".to_string()),
        path: path.to_string(),
    })
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

async fn get(path: &str) -> (StatusCode, String, Option<String>, Value) {
    get_from(app(), path).await
}

async fn get_from(app: axum::Router, path: &str) -> (StatusCode, String, Option<String>, Value) {
    let response = app
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
    let cors = response
        .headers()
        .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        .map(|value| {
            value
                .to_str()
                .expect("cors header should be ascii")
                .to_string()
        });
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let json = serde_json::from_slice(&body).expect("body should be json");
    (status, content_type, cors, json)
}

async fn raw_get_from(app: axum::Router, path: &str) -> (StatusCode, Option<String>) {
    let response = app
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    let content_type = response.headers().get(header::CONTENT_TYPE).map(|value| {
        value
            .to_str()
            .expect("content type should be ascii")
            .to_string()
    });
    (response.status(), content_type)
}

#[tokio::test]
async fn serves_autnum_with_and_without_as_prefix() {
    let (_, _, _, bare) = get("/rdap/autnum/4242423011").await;
    let (status, content_type, _, prefixed) = get("/rdap/autnum/AS4242423011").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/rdap+json");
    assert_eq!(bare["handle"], "AS4242423011");
    assert_eq!(prefixed["objectClassName"], "autnum");
    assert_eq!(prefixed["name"], "MORAXYC-AS");
    assert_eq!(prefixed["startAutnum"], 4242423011u64);
    assert_eq!(prefixed["endAutnum"], 4242423011u64);
    assert_eq!(prefixed["entities"][0]["objectClassName"], "entity");
    assert_eq!(
        prefixed["links"][0]["href"],
        "https://rdap.example.dn42/rdap/autnum/AS4242423011"
    );
    assert_eq!(
        bare["links"][0]["value"],
        "https://rdap.example.dn42/rdap/autnum/4242423011"
    );
    assert_eq!(
        bare["links"][0]["href"],
        "https://rdap.example.dn42/rdap/autnum/AS4242423011"
    );
}

#[tokio::test]
async fn serves_domain_after_lowercasing() {
    let (status, _, _, json) = get("/rdap/domain/MORAXYC.DN42").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "domain");
    assert_eq!(json["handle"], "moraxyc.dn42");
    assert_eq!(json["ldhName"], "moraxyc.dn42");
    assert_eq!(json["status"][0], "active");
    assert_eq!(
        json["remarks"]
            .as_array()
            .expect("remarks should be an array")
            .iter()
            .filter(|remark| remark["title"].as_str() == Some("descr"))
            .count(),
        1
    );
}

#[tokio::test]
async fn serves_person_entity() {
    let (status, _, _, json) = get("/rdap/entity/MORAXYC-DN42").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "entity");
    assert_eq!(json["handle"], "MORAXYC-DN42");
    assert_eq!(json["notices"][0]["title"], "Service Notice");
    assert_eq!(json["vcardArray"][1][0][3], "Moraxyc");
}

#[tokio::test]
async fn serves_ip_network_with_route_remark() {
    let (status, _, _, json) = get("/rdap/ip/172.21.86.193").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "ip network");
    assert_eq!(json["handle"], "172.21.86.192_27");
    assert_eq!(json["startAddress"], "172.21.86.192");
    assert_eq!(json["endAddress"], "172.21.86.223");
    assert_eq!(json["ipVersion"], "v4");
    assert_eq!(json["remarks"][0]["title"], "route");
}

#[tokio::test]
async fn serves_rdap_ipv6_prefix_network() {
    let (status, _, _, json) = get("/rdap/ip/fdea:a10b:3d3a::/48").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["objectClassName"], "ip network");
    assert_eq!(json["handle"], "fdea:a10b:3d3a::_48");
    assert_eq!(json["startAddress"], "fdea:a10b:3d3a::");
    assert_eq!(json["ipVersion"], "v6");
    assert_eq!(json["remarks"][0]["title"], "route6");
}

#[tokio::test]
async fn serves_ip_prefix_after_normalizing_address() {
    let (v4_status, _, _, v4_json) = get("/rdap/ip/172.21.86.193/27").await;
    let (v6_status, _, _, v6_json) = get("/rdap/ip/FDEA:A10B:3D3A:0:0:0:0:1/48").await;

    assert_eq!(v4_status, StatusCode::OK);
    assert_eq!(v4_json["handle"], "172.21.86.192_27");
    assert_eq!(
        v4_json["links"][0]["href"],
        "https://rdap.example.dn42/rdap/ip/172.21.86.192/27"
    );
    assert_eq!(v6_status, StatusCode::OK);
    assert_eq!(v6_json["handle"], "fdea:a10b:3d3a::_48");
    assert_eq!(
        v6_json["links"][0]["href"],
        "https://rdap.example.dn42/rdap/ip/fdea:a10b:3d3a::/48"
    );
}

#[tokio::test]
async fn invalid_ip_prefix_lengths_return_rdap_error() {
    let (v4_status, v4_content_type, _, v4_json) = get("/rdap/ip/172.21.86.192/33").await;
    let (v6_status, v6_content_type, _, v6_json) = get("/rdap/ip/fdea:a10b:3d3a::/129").await;

    assert_eq!(v4_status, StatusCode::BAD_REQUEST);
    assert_eq!(v4_content_type, "application/rdap+json");
    assert_eq!(v4_json["errorCode"], 400);
    assert_eq!(v6_status, StatusCode::BAD_REQUEST);
    assert_eq!(v6_content_type, "application/rdap+json");
    assert_eq!(v6_json["errorCode"], 400);
}

#[tokio::test]
async fn invalid_autnum_returns_rdap_error() {
    let (status, content_type, _, json) = get("/rdap/autnum/not-an-asn").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(content_type, "application/rdap+json");
    assert_eq!(json["errorCode"], 400);
}

#[tokio::test]
async fn missing_object_returns_rdap_error() {
    let (status, _, _, json) = get("/rdap/autnum/4242423999").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(json["errorCode"], 404);
}

#[tokio::test]
async fn unmatched_rdap_prefix_paths_return_rdap_error() {
    for path in [
        "/rdap/nameserver/ns.example",
        "/rdap/help",
        "/rdap/whatever",
    ] {
        let (status, content_type, _, json) = get(path).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(content_type, "application/rdap+json");
        assert_eq!(json["errorCode"], 404);
    }
}

#[tokio::test]
async fn unmatched_non_rdap_prefix_path_uses_outer_404() {
    let (status, content_type) = raw_get_from(app(), "/not-rdap").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_ne!(content_type.as_deref(), Some("application/rdap+json"));
}

#[tokio::test]
async fn lookup_io_error_returns_rdap_500() {
    let data_path = temp_registry_path("rdap-lookup-error");
    fs::create_dir_all(data_path.join("aut-num").join("AS4242423011"))
        .expect("directory object path should be created");
    let app = routes(RdapState {
        registry: Registry::new(data_path.clone()),
        base_url: Some("https://rdap.example.dn42".to_string()),
        path: "/rdap".to_string(),
    });

    let (status, content_type, _, json) = get_from(app, "/rdap/autnum/4242423011").await;
    let _ = fs::remove_dir_all(data_path);

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(content_type, "application/rdap+json");
    assert_eq!(json["errorCode"], 500);
}

#[tokio::test]
async fn default_rdap_response_has_cors_and_self_links() {
    let app = routes(RdapState {
        registry: Registry::new(PathBuf::from("resources/fixtures/registry-3011/data")),
        base_url: None,
        path: "/".to_string(),
    });

    let (status, _, cors, json) = get_from(app, "/domain/MORAXYC.DN42").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(cors.as_deref(), Some("*"));
    assert_eq!(json["links"][0]["href"], "/domain/moraxyc.dn42");
    assert_eq!(
        json["entities"][0]["links"][0]["href"],
        "/entity/MORAXYC-DN42"
    );
}

#[tokio::test]
async fn root_path_router_unmatched_path_returns_rdap_error() {
    let (status, content_type, _, json) = get_from(app_with_path("/"), "/whatever").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(content_type, "application/rdap+json");
    assert_eq!(json["errorCode"], 404);
}

#[tokio::test]
async fn healthz_returns_ok_with_nested_rdap_path() {
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .expect("content type should exist")
            .to_str()
            .expect("content type should be ascii"),
        "text/plain; charset=utf-8"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn healthz_works_with_root_rdap_path() {
    let response = app_with_path("/")
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("request should run");
    assert_eq!(response.status(), StatusCode::OK);
}
