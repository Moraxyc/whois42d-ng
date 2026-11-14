#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::collections::HashMap;
#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::net::{TcpListener, TcpStream};
#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::os::fd::{AsRawFd, IntoRawFd};
#[cfg(all(target_os = "linux", feature = "systemd"))]
use std::os::linux::net::SocketAddrExt;

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
fn rejects_non_tcp_listeners_without_taking_ownership() {
    let udp = std::net::UdpSocket::bind("127.0.0.1:0").expect("UDP socket should bind");
    let unix = std::os::unix::net::UnixListener::bind_addr(
        &std::os::unix::net::SocketAddr::from_abstract_name(format!(
            "whois42d-ng-test-{}",
            std::process::id()
        ))
        .expect("abstract address should be valid"),
    )
    .expect("Unix listener should bind");

    for fd in [udp.as_raw_fd(), unix.as_raw_fd()] {
        let err = whois42d_ng::socket_activation::tcp_listener_from_fd(fd)
            .expect_err("non-TCP descriptor must be rejected");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
    udp.local_addr().expect("rejected UDP socket remains open");
    unix.local_addr()
        .expect("rejected Unix socket remains open");
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
#[cfg(all(target_os = "linux", feature = "systemd"))]
fn notify_ready_sends_systemd_readiness() {
    use std::{os::unix::net::UnixDatagram, process::Command, time::Duration};

    let address = std::os::unix::net::SocketAddr::from_abstract_name(format!(
        "whois42d-ng-notify-test-{}",
        std::process::id()
    ))
    .expect("abstract address should be valid");
    let socket = UnixDatagram::bind_addr(&address).expect("notification socket should bind");
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .expect("read timeout should be set");
    let status = Command::new(std::env::current_exe().expect("test executable should exist"))
        .args(["--exact", "notify_ready_child"])
        .env_clear()
        .env("WHOIS42D_NOTIFY_TEST_CHILD", "1")
        .env(
            "NOTIFY_SOCKET",
            format!("@whois42d-ng-notify-test-{}", std::process::id()),
        )
        .status()
        .expect("notification child should run");
    assert!(status.success());
    let mut message = [0; 64];
    let size = socket
        .recv(&mut message)
        .expect("READY notification should arrive");
    assert_eq!(
        std::str::from_utf8(&message[..size]).unwrap().trim(),
        "READY=1"
    );
}

#[test]
#[cfg(all(target_os = "linux", feature = "systemd"))]
fn notify_ready_child() {
    if std::env::var_os("WHOIS42D_NOTIFY_TEST_CHILD").is_some() {
        whois42d_ng::socket_activation::notify_ready().expect("ready notification should succeed");
    }
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
