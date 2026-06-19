#[cfg(unix)]
use std::os::fd::RawFd;

#[cfg(unix)]
fn invalid_activation_fd(fd: RawFd) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("invalid socket activation fd {fd}"),
    )
}

#[cfg(unix)]
pub fn tcp_listener_from_fd(fd: RawFd) -> std::io::Result<std::net::TcpListener> {
    systemd::daemon::tcp_listener(fd).map_err(|_| invalid_activation_fd(fd))
}

#[cfg(not(unix))]
pub fn tcp_listener_from_fd(_fd: i32) -> std::io::Result<std::net::TcpListener> {
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "socket activation is only supported on unix",
    ))
}

#[cfg(unix)]
pub fn tcp_listeners_from_env() -> std::io::Result<Vec<std::net::TcpListener>> {
    systemd::daemon::listen_fds(false)
        .map_err(std::io::Error::other)?
        .iter()
        .map(tcp_listener_from_fd)
        .collect()
}

#[cfg(unix)]
pub fn notify_ready() -> std::io::Result<()> {
    systemd::daemon::notify(false, std::iter::once(&(systemd::daemon::STATE_READY, "1")))
        .map(|_| ())
        .map_err(std::io::Error::other)
}

#[cfg(not(unix))]
pub fn notify_ready() -> std::io::Result<()> {
    Ok(())
}

#[cfg(not(unix))]
pub fn tcp_listeners_from_env() -> std::io::Result<Vec<std::net::TcpListener>> {
    Ok(Vec::new())
}
