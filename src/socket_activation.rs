use std::io;

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
    systemd::daemon::listen_fds(false)
        .map_err(io::Error::other)?
        .iter()
        .map(tcp_listener_from_fd)
        .collect()
}

#[cfg(not(all(feature = "systemd", unix)))]
pub fn tcp_listeners_from_env() -> io::Result<Vec<std::net::TcpListener>> {
    Ok(Vec::new())
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
