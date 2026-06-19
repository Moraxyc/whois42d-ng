use whois42d_ng::protocol::{MAX_QUERY_LEN, QueryLineError, query_log_text, read_query_line};

#[test]
fn reads_crlf_terminated_query_without_line_ending() {
    let query = read_query_line(b"AS4242423011\r\nextra").expect("query should parse");

    assert_eq!(query, "AS4242423011");
}

#[test]
fn reads_lf_terminated_query_without_line_ending() {
    let query = read_query_line(b"-q types\n").expect("query should parse");

    assert_eq!(query, "-q types");
}

#[test]
fn rejects_overlong_query_lines() {
    let input = vec![b'a'; whois42d_ng::protocol::MAX_QUERY_LEN + 1];

    assert_eq!(read_query_line(&input), Err(QueryLineError::TooLong));
}

#[test]
fn accepts_max_length_query_with_crlf() {
    let mut input = vec![b'a'; MAX_QUERY_LEN];
    input.extend_from_slice(b"\r\n");

    let query = read_query_line(&input).expect("max length CRLF query should parse");

    assert_eq!(query.len(), MAX_QUERY_LEN);
}

#[test]
fn query_log_text_trims_and_limits_logged_input() {
    let input = format!("  {}  ", "a".repeat(MAX_QUERY_LEN + 10));
    let logged = query_log_text(&input);

    assert_eq!(logged.len(), 80 + "...".len());
    assert!(logged.starts_with(&"a".repeat(80)));
    assert!(logged.ends_with("..."));
    assert!(!logged.starts_with(' '));
    assert!(!logged.ends_with(' '));
}
