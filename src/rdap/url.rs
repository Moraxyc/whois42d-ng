use std::io;

pub fn rdap_base_url(base_url: Option<&str>, path: &str) -> String {
    match base_url.filter(|value| !value.is_empty()) {
        Some(base_url) => external_rdap_base_url(base_url, path),
        None => path_prefix(path).to_string(),
    }
}

pub fn absolute_rdap_base_url(base_url: Option<&str>, path: &str) -> io::Result<String> {
    let Some(base_url) = base_url.filter(|value| !value.is_empty()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bootstrap registry requires an absolute --rdap-base-url",
        ));
    };
    if !is_absolute_http_url(base_url) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--rdap-base-url must be an absolute http(s) URL",
        ));
    }
    Ok(external_rdap_base_url(base_url, path))
}

fn external_rdap_base_url(base_url: &str, path: &str) -> String {
    let base_url = base_url.trim_end_matches('/');
    let prefix = path_prefix(path);
    if prefix.is_empty() || base_url.ends_with(prefix) {
        base_url.to_string()
    } else {
        format!("{base_url}{prefix}")
    }
}

fn path_prefix(path: &str) -> &str {
    if path == "/" {
        ""
    } else {
        path.trim_end_matches('/')
    }
}

fn is_absolute_http_url(value: &str) -> bool {
    if value.contains('?') || value.contains('#') {
        return false;
    }
    let Some(rest) = value
        .strip_prefix("https://")
        .or_else(|| value.strip_prefix("http://"))
    else {
        return false;
    };
    rest.split('/').next().is_some_and(|host| !host.is_empty())
}
