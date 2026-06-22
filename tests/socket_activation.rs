#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::net::{TcpListener, TcpStream};
#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::os::fd::{AsRawFd, IntoRawFd};

#[test]
fn no_matching_activation_environment_yields_no_listeners() {
    let listeners = whois42d_ng::socket_activation::tcp_listeners_from_env()
        .expect("missing activation env should not error");

    assert!(listeners.is_empty());
}

#[test]
#[cfg(all(target_os = "linux", feature = "systemd"))]
fn imports_listening_tcp_fd() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let imported = whois42d_ng::socket_activation::tcp_listener_from_fd(listener.into_raw_fd())
        .expect("fd should import");

    let _client = TcpStream::connect(addr).expect("client should connect");
    imported.accept().expect("imported listener should accept");
}

#[test]
fn invalid_activation_fd_returns_clear_error() {
    let err = whois42d_ng::socket_activation::tcp_listener_from_fd(-1)
        .expect_err("invalid fd should error");

    if cfg!(all(target_os = "linux", feature = "systemd")) {
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
        assert!(err.to_string().contains("invalid socket activation fd -1"));
    } else {
        assert_eq!(err.kind(), std::io::ErrorKind::Unsupported);
    }
}

#[test]
#[cfg(all(target_os = "linux", feature = "systemd"))]
fn connected_stream_fd_returns_clear_error() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should exist");
    let stream = TcpStream::connect(addr).expect("stream should connect");
    let (_server_stream, _) = listener.accept().expect("server stream should accept");

    let err = whois42d_ng::socket_activation::tcp_listener_from_fd(stream.as_raw_fd())
        .expect_err("stream is not a listener");

    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    assert!(err.to_string().contains(&format!(
        "invalid socket activation fd {}",
        stream.as_raw_fd()
    )));
}
