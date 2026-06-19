pub const MAX_QUERY_LEN: usize = 1024;
const MAX_LOGGED_QUERY_LEN: usize = 80;

pub fn query_log_text(query: &str) -> String {
    let query = query.trim();
    if query.len() <= MAX_LOGGED_QUERY_LEN {
        return query.to_string();
    }

    format!("{}...", &query[..MAX_LOGGED_QUERY_LEN])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryLineError {
    TooLong,
    Empty,
    InvalidUtf8,
}

pub fn read_query_line(input: &[u8]) -> Result<String, QueryLineError> {
    let line_end = input
        .iter()
        .position(|byte| *byte == b'\n')
        .unwrap_or(input.len());

    let mut line = &input[..line_end];
    if line.ends_with(b"\r") {
        line = &line[..line.len() - 1];
    }
    if line.len() > MAX_QUERY_LEN {
        return Err(QueryLineError::TooLong);
    }
    if line.is_empty() {
        return Err(QueryLineError::Empty);
    }

    String::from_utf8(line.to_vec()).map_err(|_| QueryLineError::InvalidUtf8)
}
