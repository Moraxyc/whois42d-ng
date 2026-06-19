use crate::types::ALL_TYPES;

pub fn server_info(what: &str) -> String {
    match what {
        "version" => "% whois42d v1\n".to_string(),
        "sources" => "DN42:3:N:0-0\n".to_string(),
        "types" => ALL_TYPES.join("\n") + "\n",
        other => format!("% unknown option {other}\n"),
    }
}

pub fn not_found() -> &'static str {
    "% 404\n"
}
