use std::{collections::HashMap, io};

#[cfg(all(feature = "systemd", unix))]
use std::os::fd::RawFd;

#[cfg(all(feature = "systemd", unix))]
fn invalid_activation_fd(fd: RawFd) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid socket activation fd {fd}"),
    )
}

#[cfg(all(feature = "systemd", unix))]
pub fn tcp_listener_from_fd(fd: RawFd) -> io::Result<std::net::TcpListener> {
    systemd::daemon::tcp_listener(fd).map_err(|_| invalid_activation_fd(fd))
}

#[cfg(all(not(feature = "systemd"), unix))]
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

#[cfg(all(feature = "systemd", unix))]
pub fn tcp_listeners_from_env() -> io::Result<Vec<std::net::TcpListener>> {
    listener_fds_by_name()?
        .remove(&None)
        .unwrap_or_default()
        .into_iter()
        .map(tcp_listener_from_fd)
        .collect()
}

#[cfg(not(all(feature = "systemd", unix)))]
pub fn tcp_listeners_from_env() -> io::Result<Vec<std::net::TcpListener>> {
    Ok(Vec::new())
}

#[cfg(all(feature = "systemd", unix))]
pub fn listener_fds_by_name() -> io::Result<HashMap<Option<String>, Vec<RawFd>>> {
    let fds = systemd::daemon::listen_fds(false).map_err(io::Error::other)?;
    let names = std::env::var("LISTEN_FDNAMES").unwrap_or_default();
    let mut names = names.split(':');
    let mut grouped: HashMap<Option<String>, Vec<RawFd>> = HashMap::new();

    for fd in fds.iter() {
        let name = names
            .next()
            .filter(|name| !name.is_empty())
            .map(str::to_string);
        grouped.entry(name).or_default().push(fd);
    }
    Ok(grouped)
}

#[cfg(not(all(feature = "systemd", unix)))]
pub fn listener_fds_by_name() -> io::Result<HashMap<Option<String>, Vec<i32>>> {
    Ok(HashMap::new())
}

#[cfg(all(feature = "systemd", unix))]
pub fn notify_ready() -> io::Result<()> {
    systemd::daemon::notify(false, std::iter::once(&(systemd::daemon::STATE_READY, "1")))
        .map(|_| ())
        .map_err(io::Error::other)
}

#[cfg(not(all(feature = "systemd", unix)))]
pub fn notify_ready() -> io::Result<()> {
    Ok(())
}
