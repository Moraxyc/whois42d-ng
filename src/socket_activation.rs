pub fn activation_fds(
    listen_pid: Option<u32>,
    listen_fds: Option<u32>,
    current_pid: u32,
) -> Vec<i32> {
    if listen_pid != Some(current_pid) {
        return Vec::new();
    }

    let Some(count) = listen_fds else {
        return Vec::new();
    };
    (3..3 + count as i32).collect()
}

pub fn activation_fds_from_env() -> Vec<i32> {
    let listen_pid = std::env::var("LISTEN_PID")
        .ok()
        .and_then(|value| value.parse().ok());
    let listen_fds = std::env::var("LISTEN_FDS")
        .ok()
        .and_then(|value| value.parse().ok());
    activation_fds(listen_pid, listen_fds, std::process::id())
}

#[cfg(unix)]
pub fn tcp_listeners_from_fds(fds: &[i32]) -> std::io::Result<Vec<std::net::TcpListener>> {
    use std::io;
    use std::mem;
    use std::os::fd::FromRawFd;

    let mut listeners = Vec::with_capacity(fds.len());
    for fd in fds {
        let mut socket_type = 0;
        let mut len = mem::size_of_val(&socket_type) as libc::socklen_t;
        let rc = unsafe {
            libc::getsockopt(
                *fd,
                libc::SOL_SOCKET,
                libc::SO_TYPE,
                &mut socket_type as *mut _ as *mut libc::c_void,
                &mut len,
            )
        };
        if rc != 0 || socket_type != libc::SOCK_STREAM {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid socket activation fd {fd}"),
            ));
        }
        listeners.push(unsafe { std::net::TcpListener::from_raw_fd(*fd) });
    }
    Ok(listeners)
}

#[cfg(not(unix))]
pub fn tcp_listeners_from_fds(_fds: &[i32]) -> std::io::Result<Vec<std::net::TcpListener>> {
    Ok(Vec::new())
}

#[cfg(unix)]
pub fn tcp_listeners_from_env() -> std::io::Result<Vec<std::net::TcpListener>> {
    tcp_listeners_from_fds(&activation_fds_from_env())
}

#[cfg(not(unix))]
pub fn tcp_listeners_from_env() -> std::io::Result<Vec<std::net::TcpListener>> {
    Ok(Vec::new())
}
