use whois42d_ng::socket_activation::{activation_fds, tcp_listeners_from_fds};

#[test]
fn imports_systemd_fds_when_pid_matches() {
    let pid = std::process::id();

    assert_eq!(activation_fds(Some(pid), Some(2), pid), vec![3, 4]);
}

#[test]
fn ignores_systemd_fds_when_pid_does_not_match() {
    assert!(activation_fds(Some(1), Some(2), 2).is_empty());
}

#[test]
fn invalid_activation_fd_returns_clear_error() {
    let err = tcp_listeners_from_fds(&[-1]).expect_err("invalid fd should error");

    assert!(err.to_string().contains("invalid socket activation fd -1"));
}
