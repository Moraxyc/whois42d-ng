use std::net::TcpListener;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use whois42d_ng::registry::Registry;
use whois42d_ng::server::{Cli, CliCommand, Options, serve_listener_until_idle};
use whois42d_ng::signals::shutdown_flag;
use whois42d_ng::socket_activation::{notify_ready, tcp_listeners_from_env};

fn main() -> ExitCode {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            log::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> std::io::Result<()> {
    let cli = Cli::parse();
    if matches!(cli.command, Some(CliCommand::Completions { .. })) {
        cli.write_completions(&mut std::io::stdout());
        return Ok(());
    }

    let options = cli.options;
    run_daemon(options)
}

fn run_daemon(options: Options) -> std::io::Result<()> {
    log::info!("starting whois42d-ng daemon");
    let data_path = options.registry_data_path()?;
    log::info!("serving registry from {}", data_path.display());
    let registry = Registry::new(data_path);
    let listeners = tcp_listeners_from_env()?;
    let shutdown = shutdown_flag()?;

    if listeners.is_empty() {
        let listen_addr = options.listen_addr();
        log::info!("binding to {listen_addr}");
        let listener = TcpListener::bind(listen_addr)?;
        notify_ready()?;
        serve_listener_until_idle(listener, registry, std::time::Duration::MAX, shutdown)
    } else {
        log::info!("socket activation: {} listener(s)", listeners.len());
        log::info!("socket activation idle timeout: {:?}", options.timeout);
        notify_ready()?;
        let mut workers = Vec::new();
        for listener in listeners {
            let registry = registry.clone();
            let shutdown = Arc::clone(&shutdown);
            let timeout = options.timeout;
            workers.push(std::thread::spawn(move || {
                serve_listener_until_idle(listener, registry, timeout, shutdown)
            }));
        }
        for worker in workers {
            worker
                .join()
                .map_err(|_| std::io::Error::other("listener worker panicked"))??;
        }
        Ok(())
    }
}
