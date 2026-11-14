use std::{collections::HashMap, io};

#[cfg(all(feature = "systemd", target_os = "linux"))]
use std::os::fd::{BorrowedFd, FromRawFd, RawFd};

#[cfg(all(feature = "systemd", target_os = "linux"))]
fn invalid_activation_fd(fd: RawFd) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid socket activation fd {fd}"),
    )
}

#[cfg(all(feature = "systemd", target_os = "linux"))]
pub fn tcp_listener_from_fd(fd: RawFd) -> io::Result<std::net::TcpListener> {
    if fd < 0 {
        return Err(invalid_activation_fd(fd));
    }
    // Borrow for validation; ownership transfers only after confirming a TCP listener.
    let borrowed = unsafe { BorrowedFd::borrow_raw(fd) };
    let socket = socket2::SockRef::from(&borrowed);
    let valid = (|| -> io::Result<bool> {
        Ok(socket.r#type()? == socket2::Type::STREAM
            && socket.is_listener()?
            && socket.local_addr()?.as_socket().is_some())
    })();
    if !valid.unwrap_or(false) {
        return Err(invalid_activation_fd(fd));
    }
    Ok(unsafe { std::net::TcpListener::from_raw_fd(fd) })
}

#[cfg(all(not(all(feature = "systemd", target_os = "linux")), unix))]
pub fn tcp_listener_from_fd(_fd: std::os::unix::io::RawFd) -> io::Result<std::net::TcpListener> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "built without systemd support",
    ))
}

#[cfg(not(unix))]
pub fn tcp_listener_from_fd(_fd: i32) -> io::Result<std::net::TcpListener> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "socket activation is only supported on unix",
    ))
}

#[cfg(all(feature = "systemd", target_os = "linux"))]
pub fn tcp_listeners_from_env() -> io::Result<Vec<std::net::TcpListener>> {
    let (whois_fds, _) = split_listener_fds_by_role(listener_fds_by_name()?);
    whois_fds.into_iter().map(tcp_listener_from_fd).collect()
}

#[cfg(not(all(feature = "systemd", target_os = "linux")))]
pub fn tcp_listeners_from_env() -> io::Result<Vec<std::net::TcpListener>> {
    Ok(Vec::new())
}

#[cfg(all(feature = "systemd", target_os = "linux"))]
pub fn listener_fds_by_name() -> io::Result<HashMap<Option<String>, Vec<RawFd>>> {
    let fds = sd_notify::listen_fds()?;
    let names = std::env::var("LISTEN_FDNAMES").unwrap_or_default();
    let mut names = names.split(':');
    let mut grouped: HashMap<Option<String>, Vec<RawFd>> = HashMap::new();

    for fd in fds {
        let name = names
            .next()
            .filter(|name| !name.is_empty())
            .map(str::to_string);
        grouped.entry(name).or_default().push(fd);
    }
    Ok(grouped)
}

#[cfg(all(feature = "systemd", target_os = "linux"))]
pub fn split_listener_fds_by_role(
    mut activation: HashMap<Option<String>, Vec<RawFd>>,
) -> (Vec<RawFd>, Vec<RawFd>) {
    let rdap_fds = activation
        .remove(&Some("rdap".to_string()))
        .unwrap_or_default();
    let whois_fds = activation.into_values().flatten().collect();

    (whois_fds, rdap_fds)
}

#[cfg(not(all(feature = "systemd", target_os = "linux")))]
pub fn listener_fds_by_name() -> io::Result<HashMap<Option<String>, Vec<i32>>> {
    Ok(HashMap::new())
}

#[cfg(not(all(feature = "systemd", target_os = "linux")))]
pub fn split_listener_fds_by_role(
    activation: HashMap<Option<String>, Vec<i32>>,
) -> (Vec<i32>, Vec<i32>) {
    let _ = activation;
    (Vec::new(), Vec::new())
}

#[cfg(all(feature = "systemd", target_os = "linux"))]
pub fn notify_ready() -> io::Result<()> {
    use std::os::{linux::net::SocketAddrExt, unix::ffi::OsStrExt, unix::net};

    let Some(path) = std::env::var_os("NOTIFY_SOCKET") else {
        return Ok(());
    };
    // sd-notify 0.5 treats @name as a pathname, not a Linux abstract socket.
    let address = match path.as_bytes().strip_prefix(b"@") {
        Some(name) => net::SocketAddr::from_abstract_name(name)?,
        None => net::SocketAddr::from_pathname(path)?,
    };
    let socket = net::UnixDatagram::unbound()?;
    socket.connect_addr(&address)?;
    if socket.send(b"READY=1")? != b"READY=1".len() {
        return Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "incomplete READY notification",
        ));
    }
    Ok(())
}

#[cfg(not(all(feature = "systemd", target_os = "linux")))]
pub fn notify_ready() -> io::Result<()> {
    Ok(())
}
