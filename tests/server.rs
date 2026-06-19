use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, atomic::AtomicBool, mpsc};
use std::time::Duration;

use whois42d_ng::registry::Registry;
use whois42d_ng::server::{serve_listener_until_idle, serve_one};

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

#[test]
fn socket_activation_listener_exits_after_idle_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let shutdown = Arc::new(AtomicBool::new(false));

    serve_listener_until_idle(listener, registry, Duration::from_millis(1), shutdown)
        .expect("idle listener should exit cleanly");
}

#[test]
fn listener_exits_when_shutdown_flag_is_set() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let shutdown = Arc::new(AtomicBool::new(true));

    serve_listener_until_idle(listener, registry, Duration::ZERO, shutdown)
        .expect("shutdown listener should exit cleanly");
}

#[test]
fn zero_idle_timeout_does_not_exit_before_shutdown() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let registry = Registry::new(PathBuf::from("resources/fixtures/registry-3011/data"));
    let shutdown = Arc::new(AtomicBool::new(false));
    let worker_shutdown = Arc::clone(&shutdown);
    let (done_tx, done_rx) = mpsc::channel();

    std::thread::spawn(move || {
        let result = serve_listener_until_idle(listener, registry, Duration::ZERO, worker_shutdown);
        done_tx.send(result).expect("done should send");
    });

    let mut client = TcpStream::connect(addr).expect("client should connect");
    client
        .write_all(b"AS4242423011\n")
        .expect("query should write");
    client
        .shutdown(Shutdown::Write)
        .expect("write side should close");
    let mut response = String::new();
    client
        .read_to_string(&mut response)
        .expect("response should read");
    shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

    assert!(response.contains("aut-num:            AS4242423011"));
    done_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("server should exit after shutdown")
        .expect("server should exit cleanly");
}
