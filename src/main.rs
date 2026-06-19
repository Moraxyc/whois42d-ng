use std::net::TcpListener;
use std::process::ExitCode;
use std::sync::Arc;

use clap::Parser;
use whois42d_ng::registry::Registry;
use whois42d_ng::server::{Cli, CliCommand, Options, serve_listener_until_idle};
use whois42d_ng::signals::shutdown_flag;
use whois42d_ng::socket_activation::tcp_listeners_from_env;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Error: {err}");
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
    let registry = Registry::new(options.registry_data_path()?);
    let listeners = tcp_listeners_from_env()?;
    let shutdown = shutdown_flag()?;

    if listeners.is_empty() {
        serve_listener_until_idle(
            TcpListener::bind(options.listen_addr())?,
            registry,
            std::time::Duration::MAX,
            shutdown,
        )
    } else {
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
