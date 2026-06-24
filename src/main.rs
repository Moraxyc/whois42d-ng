use clap::Parser;
use std::net::TcpListener;
use std::process::ExitCode;
use whois42d_ng::rdap::http::{RdapState, routes};
use whois42d_ng::registry::Registry;
use whois42d_ng::server::{Cli, CliCommand, Options, serve_listener_until_idle_async};
use whois42d_ng::signals::shutdown_flag;
use whois42d_ng::socket_activation::{listener_fds_by_name, notify_ready, tcp_listener_from_fd};

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
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(run_daemon(options))
}

async fn run_daemon(options: Options) -> std::io::Result<()> {
    log::info!("starting whois42d-ng daemon");
    let data_path = options.registry_data_path()?;
    log::info!("serving registry from {}", data_path.display());
    let registry = Registry::new(data_path);
    let mut activation = listener_fds_by_name()?;
    let mut listeners: Vec<TcpListener> = activation
        .remove(&None)
        .unwrap_or_default()
        .into_iter()
        .map(tcp_listener_from_fd)
        .collect::<std::io::Result<_>>()?;
    let mut rdap_listeners: Vec<TcpListener> = activation
        .remove(&Some("rdap".to_string()))
        .unwrap_or_default()
        .into_iter()
        .map(tcp_listener_from_fd)
        .collect::<std::io::Result<_>>()?;
    let activated = !listeners.is_empty();
    let shutdown = shutdown_flag()?;

    if !activated {
        let listen_addr = options.listen_addr();
        log::info!("binding to {listen_addr}");
        listeners.push(TcpListener::bind(listen_addr)?);
    } else {
        log::info!("socket activation: {} listener(s)", listeners.len());
        log::info!("socket activation idle timeout: {:?}", options.timeout);
    }

    if !rdap_listeners.is_empty() && options.rdap_port != 0 {
        log::warn!("socket activation RDAP listener overrides --rdap-port");
    } else if rdap_listeners.is_empty() && options.rdap_port != 0 {
        let listen_addr = options.rdap_listen_addr();
        log::info!("binding RDAP to {listen_addr}");
        rdap_listeners.push(TcpListener::bind(listen_addr)?);
    }

    let mut tokio_listeners = Vec::new();
    for listener in listeners {
        listener.set_nonblocking(true)?;
        tokio_listeners.push(tokio::net::TcpListener::from_std(listener)?);
    }
    let mut tokio_rdap_listeners = Vec::new();
    for listener in rdap_listeners {
        listener.set_nonblocking(true)?;
        tokio_rdap_listeners.push(tokio::net::TcpListener::from_std(listener)?);
    }

    notify_ready()?;

    let rdap_router = routes(RdapState {
        registry: registry.clone(),
        base_url: (!options.rdap_base_url.is_empty()).then_some(options.rdap_base_url.clone()),
        path: options.rdap_path.clone(),
    });
    let mut rdap_workers = Vec::new();
    for listener in tokio_rdap_listeners {
        rdap_workers.push(tokio::spawn(
            axum::serve(listener, rdap_router.clone()).into_future(),
        ));
    }

    if !activated {
        let whois = serve_listener_until_idle_async(
            tokio_listeners.remove(0),
            registry,
            std::time::Duration::MAX,
            shutdown,
        )
        .await;
        for worker in rdap_workers {
            worker.abort();
        }
        whois
    } else {
        let mut workers = Vec::new();
        for listener in tokio_listeners {
            let registry = registry.clone();
            let shutdown = shutdown.clone();
            let timeout = options.timeout;
            workers.push(tokio::spawn(serve_listener_until_idle_async(
                listener, registry, timeout, shutdown,
            )));
        }
        for worker in workers {
            worker
                .await
                .map_err(|_| std::io::Error::other("listener worker panicked"))??;
        }
        for worker in rdap_workers {
            worker.abort();
        }
        Ok(())
    }
}
