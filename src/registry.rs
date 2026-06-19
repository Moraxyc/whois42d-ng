use std::cmp::Reverse;
use std::fs;
use std::io;
use std::net::IpAddr;
use std::path::{Component, Path, PathBuf};

use crate::cidr::RegistryCidr;
use crate::response;
use crate::types::candidate_types;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    pub server_info: Option<String>,
    pub type_filter: Vec<String>,
    pub type_schema: Option<String>,
    pub objects: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    data_path: PathBuf,
}

impl Registry {
    pub fn new(data_path: PathBuf) -> Self {
        Self { data_path }
    }

    pub fn handle_query(&self, input: &str) -> io::Result<String> {
        let query = match Query::parse(input) {
            Ok(query) => query,
            Err(_) => {
                return Ok(
                    "% This is the dn42 whois query service.\n\n% error: invalid query\n\n"
                        .to_string(),
                );
            }
        };
        let mut output = "% This is the dn42 whois query service.\n\n".to_string();

        if let Some(server_info) = query.server_info {
            output.push_str(&response::server_info(&server_info));
            return Ok(output);
        }

        if let Some(type_schema) = query.type_schema {
            output.push_str(&format!(
                "% template queries are unsupported for {type_schema}\n"
            ));
            return Ok(output);
        }

        let mut found = false;
        for object in query.objects {
            if self.render_object_query(&mut output, &query.type_filter, &object)? {
                found = true;
            }
        }

        if !found {
            output.push_str(response::not_found());
        }
        output.push('\n');
        Ok(output)
    }

    fn render_object_query(
        &self,
        output: &mut String,
        type_filter: &[String],
        object: &str,
    ) -> io::Result<bool> {
        let mut found = false;
        if let Some(cidr) = RegistryCidr::parse_query(object) {
            found |= self.render_matching_cidrs(output, type_filter, cidr.addr())?;
        } else if let Ok(addr) = object.parse::<IpAddr>() {
            found |= self.render_matching_cidrs(output, type_filter, addr)?;
        }

        for object_type in candidate_types(object) {
            if !type_allowed(type_filter, object_type) {
                continue;
            }
            let object_name = normalized_object_name(object_type, object);
            if self.render_file(output, object_type, &object_name)? {
                found = true;
            }
        }

        Ok(found)
    }

    fn render_matching_cidrs(
        &self,
        output: &mut String,
        type_filter: &[String],
        addr: IpAddr,
    ) -> io::Result<bool> {
        let route_types = match addr {
            IpAddr::V4(_) => ["inetnum", "route"],
            IpAddr::V6(_) => ["inet6num", "route6"],
        };
        let mut matches = Vec::new();
        for object_type in route_types {
            if !type_allowed(type_filter, object_type) {
                continue;
            }
            let dir = self.data_path.join(object_type);
            let Ok(entries) = fs::read_dir(dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let name = entry.file_name();
                let Some(name) = name.to_str() else {
                    continue;
                };
                let Some(cidr) = RegistryCidr::parse_file_name(name) else {
                    continue;
                };
                if cidr.contains(addr) {
                    matches.push((cidr.prefix(), object_type, name.to_string()));
                }
            }
        }

        matches.sort_by_key(|entry| Reverse(entry.0));
        let mut found = false;
        for (_, object_type, name) in matches {
            if self.render_file(output, object_type, &name)? {
                found = true;
            }
        }
        Ok(found)
    }

    fn render_file(
        &self,
        output: &mut String,
        object_type: &str,
        object: &str,
    ) -> io::Result<bool> {
        if unsafe_path_segment(object) {
            return Ok(false);
        }
        let path = self.data_path.join(object_type).join(object);
        match fs::read_to_string(&path) {
            Ok(contents) => {
                output.push_str(&format!(
                    "% Information related to '{object_type}/{object}':\n"
                ));
                output.push_str(&contents);
                if !contents.ends_with('\n') {
                    output.push('\n');
                }
                output.push('\n');
                Ok(true)
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(err) => {
                log::warn!("failed to read registry object {}: {err}", path.display());
                Ok(false)
            }
        }
    }
}

impl Query {
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut args = split_args(input);
        let mut query = Query {
            server_info: None,
            type_filter: Vec::new(),
            type_schema: None,
            objects: Vec::new(),
        };

        while let Some(arg) = args.first().cloned() {
            if !arg.starts_with('-') {
                break;
            }
            args.remove(0);
            let Some(value) = args.first().cloned() else {
                return Err(format!("missing value for {arg}"));
            };
            args.remove(0);

            match arg.as_str() {
                "-q" => query.server_info = Some(value),
                "-T" => {
                    query.type_filter = value
                        .split(',')
                        .filter(|part| !part.is_empty())
                        .map(str::to_string)
                        .collect();
                }
                "-t" => query.type_schema = Some(value),
                _ => return Err(format!("unsupported option {arg}")),
            }
        }

        query.objects = args;
        Ok(query)
    }
}

fn split_args(input: &str) -> Vec<String> {
    input.split_whitespace().map(str::to_string).collect()
}

fn type_allowed(type_filter: &[String], object_type: &str) -> bool {
    type_filter.is_empty() || type_filter.iter().any(|filter| filter == object_type)
}

fn normalized_object_name(object_type: &str, object: &str) -> String {
    match object_type {
        "dns" => object.to_ascii_lowercase(),
        _ => object.to_ascii_uppercase(),
    }
}

fn unsafe_path_segment(value: &str) -> bool {
    Path::new(value)
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
}
