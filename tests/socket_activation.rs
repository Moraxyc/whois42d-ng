#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::collections::HashMap;
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
#[cfg(all(target_os = "linux", feature = "systemd"))]
fn treats_non_rdap_activation_fds_as_whois() {
    let mut activation = HashMap::new();
    activation.insert(Some("whois42d-ng.socket".to_string()), vec![3, 4]);
    activation.insert(Some("rdap".to_string()), vec![5, 6]);
    activation.insert(None, vec![7]);

    let (mut whois_fds, rdap_fds) =
        whois42d_ng::socket_activation::split_listener_fds_by_role(activation);
    whois_fds.sort_unstable();

    assert_eq!(whois_fds, vec![3, 4, 7]);
    assert_eq!(rdap_fds, vec![5, 6]);
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
