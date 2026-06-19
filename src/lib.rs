pub mod cidr;
pub mod protocol;
pub mod registry;
pub mod response;
pub mod server;
pub mod signals;
pub mod socket_activation;
pub mod types;

pub const HAS_SYSTEMD: bool = cfg!(feature = "systemd");
