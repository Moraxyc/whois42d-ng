use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool, mpsc};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use whois42d_ng::registry::Registry;
use whois42d_ng::server::{serve_listener_until_idle_async, serve_one, serve_one_async};

#[test]
fn serves_one_query_and_closes_connection() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let (done_tx, done_rx) = mpsc::channel();

    std::thread::spawn(move || {
        let (stream, _) = listener.accept().expect("connection should accept");
        let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
        serve_one(stream, &registry).expect("query should serve");
        done_tx.send(()).expect("done should send");
    });

    let mut client = TcpStream::connect(addr).expect("client should connect");
    client
        .write_all(b"AS4242423011\r\nsecond query\r\n")
        .expect("query should write");
    client
        .shutdown(Shutdown::Write)
        .expect("write side should close");

    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .expect("response should read until close");

    assert!(response.contains("aut-num:            AS4242423011"));
    assert!(!response.contains("second query"));
    done_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("server should close connection");
}

#[tokio::test]
async fn serves_one_query_async_and_closes_connection() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let worker = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("connection should accept");
        serve_one_async(stream, registry)
            .await
            .expect("query should serve");
    });

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .expect("client should connect");
    client
        .write_all(b"AS4242423011\r\nsecond query\r\n")
        .await
        .expect("query should write");
    client.shutdown().await.expect("write side should close");

    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .await
        .expect("response should read until close");

    assert!(response.contains("aut-num:            AS4242423011"));
    assert!(!response.contains("second query"));
    worker.await.expect("server task should finish");
}

#[tokio::test]
async fn async_connection_read_times_out_when_client_sends_nothing() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let worker = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("connection should accept");
        serve_one_async(stream, registry).await
    });

    let _client = tokio::net::TcpStream::connect(addr)
        .await
        .expect("client should connect");

    let result = tokio::time::timeout(Duration::from_secs(6), worker)
        .await
        .expect("server should time out idle read")
        .expect("server task should finish");
    assert_eq!(
        result.expect_err("read should time out").kind(),
        ErrorKind::TimedOut
    );
}

#[tokio::test]
async fn socket_activation_listener_exits_after_idle_timeout() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let shutdown = Arc::new(AtomicBool::new(false));

    serve_listener_until_idle_async(listener, registry, Duration::from_millis(1), shutdown)
        .await
        .expect("idle listener should exit cleanly");
}

#[tokio::test]
async fn listener_exits_when_shutdown_flag_is_set() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let shutdown = Arc::new(AtomicBool::new(true));

    serve_listener_until_idle_async(listener, registry, Duration::ZERO, shutdown)
        .await
        .expect("shutdown listener should exit cleanly");
}

#[tokio::test]
async fn zero_idle_timeout_does_not_exit_before_shutdown() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let shutdown = Arc::new(AtomicBool::new(false));
    let worker_shutdown = Arc::clone(&shutdown);

    let worker = tokio::spawn(async move {
        serve_listener_until_idle_async(listener, registry, Duration::ZERO, worker_shutdown).await
    });

    let mut client = tokio::net::TcpStream::connect(addr)
        .await
        .expect("client should connect");
    client
        .write_all(b"AS4242423011\n")
        .await
        .expect("query should write");
    client.shutdown().await.expect("write side should close");
    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .await
        .expect("response should read");
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(response.contains("aut-num:            AS4242423011"));
    worker
        .await
        .expect("server should exit after shutdown")
        .expect("server should exit cleanly");
}
