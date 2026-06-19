use std::io;
use std::sync::{Arc, atomic::AtomicBool};

pub fn shutdown_flag() -> io::Result<Arc<AtomicBool>> {
    let shutdown = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&shutdown))?;
    signal_hook::flag::register(signal_hook::consts::SIGTERM, Arc::clone(&shutdown))?;
    Ok(shutdown)
}
