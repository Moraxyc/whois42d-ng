use std::io::{self, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use clap_complete::{Shell, generate};

use crate::protocol::{MAX_QUERY_LEN, QueryLineError, query_log_text, read_query_line};
use crate::registry::Registry;

#[derive(Debug, Clone, Parser, PartialEq, Eq)]
#[command(version, about = "Run a DN42 WHOIS daemon")]
pub struct Cli {
    #[command(flatten)]
    pub options: Options,
    #[command(subcommand)]
    pub command: Option<CliCommand>,
}

impl Cli {
    pub fn write_completions<W: Write>(&self, output: &mut W) {
        if let Some(CliCommand::Completions { shell }) = self.command {
            let mut command = Self::command();
            let name = command.get_name().to_string();
            generate(shell, &mut command, name, output);
        }
    }
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub enum CliCommand {
    /// Generate shell completions
    Completions { shell: Shell },
}

#[derive(Debug, Clone, clap::Args, PartialEq, Eq)]
pub struct Options {
    /// Address to bind. Use 0.0.0.0 or :: to bind all interfaces.
    #[arg(long, default_value = "", value_parser = parse_address)]
    pub address: String,
    /// TCP port to listen on.
    #[arg(long, default_value_t = 43)]
    pub port: u16,
    /// Path to the registry root.
    #[arg(long, default_value = ".")]
    #[arg(value_hint = ValueHint::DirPath)]
    pub registry: PathBuf,
    /// Socket activation idle timeout in seconds.
    #[arg(long, default_value = "10", value_parser = parse_timeout)]
    pub timeout: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            address: String::new(),
            port: 43,
            registry: PathBuf::from("."),
            timeout: Duration::from_secs(10),
        }
    }
}

impl Options {
    pub fn parse_from<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<std::ffi::OsString> + Clone,
    {
        Cli::try_parse_from(args)
            .map(|cli| cli.options)
            .map_err(|err| err.to_string())
    }

    pub fn registry_data_path(&self) -> io::Result<PathBuf> {
        let data_path = self.registry.join("data");
        if data_path.is_dir() {
            Ok(data_path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("cannot access registry data path {}", data_path.display()),
            ))
        }
    }

    pub fn listen_addr(&self) -> String {
        let addr = if self.address.is_empty() {
            "127.0.0.1"
        } else {
            &self.address
        };
        if addr.contains(':') {
            format!("[{addr}]:{}", self.port)
        } else {
            format!("{addr}:{}", self.port)
        }
    }
}

fn parse_address(value: &str) -> Result<String, String> {
    if value == "*" {
        return Err("'*' is not supported; use 0.0.0.0 or :: to bind all interfaces".to_string());
    }

    Ok(value.to_string())
}

fn parse_timeout(value: &str) -> Result<Duration, String> {
    let seconds = value.parse().map_err(|_| "invalid timeout")?;
    Ok(Duration::from_secs(seconds))
}

pub fn serve_one(mut stream: TcpStream, registry: &Registry) -> io::Result<()> {
    let started = Instant::now();
    if let Ok(peer) = stream.peer_addr() {
        log::debug!("accepted connection from {peer}");
    }

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut buffer = [0; MAX_QUERY_LEN + 2];
    let count = stream.read(&mut buffer)?;
    let response = match read_query_line(&buffer[..count]) {
        Ok(query) => {
            log::debug!("query: {}", query_log_text(&query));
            registry.handle_query(&query)?
        }
        Err(QueryLineError::TooLong) => {
            log::debug!("rejected overlong query line");
            "% error: query too long\n".to_string()
        }
        Err(_) => {
            log::debug!("rejected invalid query line");
            "% error: invalid query\n".to_string()
        }
    };
    stream.write_all(response.as_bytes())?;
    log::debug!("completed request in {:?}", started.elapsed());
    Ok(())
}

pub fn serve_listener(listener: TcpListener, registry: Registry) -> io::Result<()> {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = serve_one(stream, &registry) {
                    log::warn!("connection failed: {err}");
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

pub fn serve_listener_until_idle(
    listener: TcpListener,
    registry: Registry,
    idle_timeout: Duration,
    shutdown: Arc<AtomicBool>,
) -> io::Result<()> {
    listener.set_nonblocking(true)?;
    let mut last_connection = Instant::now();

    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => {
                last_connection = Instant::now();
                if let Err(err) = serve_one(stream, &registry) {
                    log::warn!("connection failed: {err}");
                }
            }
            Err(err) if err.kind() == ErrorKind::WouldBlock => {
                if !idle_timeout.is_zero() && last_connection.elapsed() >= idle_timeout {
                    log::info!("listener idle for {:?}, exiting", idle_timeout);
                    break;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(err) => return Err(err),
        }
    }

    if shutdown.load(Ordering::Relaxed) {
        log::info!("listener exiting: shutdown requested");
    }

    Ok(())
}
