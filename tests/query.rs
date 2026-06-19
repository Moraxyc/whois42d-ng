use whois42d_ng::registry::Query;

#[test]
fn parses_server_info_query() {
    let query = Query::parse("-q version").expect("query should parse");

    assert_eq!(query.server_info.as_deref(), Some("version"));
    assert!(query.objects.is_empty());
}

#[test]
fn parses_type_filter_and_objects() {
    let query =
        Query::parse("-T aut-num,person AS4242423011 MORAXYC-DN42").expect("query should parse");

    assert_eq!(query.type_filter, vec!["aut-num", "person"]);
    assert_eq!(query.objects, vec!["AS4242423011", "MORAXYC-DN42"]);
}
