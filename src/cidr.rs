use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryCidr {
    addr: IpAddr,
    prefix: u8,
}

impl RegistryCidr {
    pub fn parse_file_name(name: &str) -> Option<Self> {
        Self::parse_registry_name(name)
    }

    pub fn parse_query(value: &str) -> Option<Self> {
        Self::parse_registry_name(&value.replace('/', "_"))
    }

    fn parse_registry_name(name: &str) -> Option<Self> {
        let (addr, prefix) = name.split_once('_')?;
        let addr = addr.parse().ok()?;
        let prefix = prefix.parse().ok()?;
        let cidr = Self { addr, prefix };
        cidr.valid_prefix().then_some(cidr)
    }

    pub fn contains(&self, addr: IpAddr) -> bool {
        match (self.addr, addr) {
            (IpAddr::V4(network), IpAddr::V4(addr)) => contains_v4(network, addr, self.prefix),
            (IpAddr::V6(network), IpAddr::V6(addr)) => contains_v6(network, addr, self.prefix),
            _ => false,
        }
    }

    pub fn addr(&self) -> IpAddr {
        self.addr
    }

    pub fn file_name(&self) -> String {
        format!("{}_{}", self.addr, self.prefix)
    }

    pub fn prefix(&self) -> u8 {
        self.prefix
    }

    fn valid_prefix(&self) -> bool {
        match self.addr {
            IpAddr::V4(_) => self.prefix <= 32,
            IpAddr::V6(_) => self.prefix <= 128,
        }
    }
}

fn contains_v4(network: Ipv4Addr, addr: Ipv4Addr, prefix: u8) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    };
    u32::from(network) & mask == u32::from(addr) & mask
}

fn contains_v6(network: Ipv6Addr, addr: Ipv6Addr, prefix: u8) -> bool {
    let mask = if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    };
    u128::from(network) & mask == u128::from(addr) & mask
}
