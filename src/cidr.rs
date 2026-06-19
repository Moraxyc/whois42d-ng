use std::net::IpAddr;

use ipnet::IpNet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryCidr {
    net: IpNet,
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
        IpNet::new(addr, prefix).ok().map(|net| Self { net })
    }

    pub fn contains(&self, addr: IpAddr) -> bool {
        self.net.contains(&addr)
    }

    pub fn addr(&self) -> IpAddr {
        self.net.addr()
    }

    pub fn file_name(&self) -> String {
        format!("{}_{}", self.addr(), self.prefix())
    }

    pub fn prefix(&self) -> u8 {
        self.net.prefix_len()
    }
}
