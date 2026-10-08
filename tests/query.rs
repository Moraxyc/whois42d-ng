use whois42d_ng::registry::Query;

#[test]
fn parses_server_info_values_even_when_they_look_like_options() {
    for (input, expected, objects) in [
        ("-q version", "version", vec![]),
        ("-q -T AS4242423011", "-T", vec!["AS4242423011"]),
    ] {
        let query = Query::parse(input).expect("query should parse");

        assert_eq!(query.server_info.as_deref(), Some(expected), "{input}");
        assert_eq!(query.objects, objects, "{input}");
    }
}

#[test]
fn parses_leading_options_without_consuming_object_arguments() {
    let query = Query::parse(
        "  -T dns -q version -t person -q sources -T aut-num,,person,\nAS4242423011\tMORAXYC-DN42 -q version  ",
    )
    .expect("query should parse");

    assert_eq!(query.server_info.as_deref(), Some("sources"));
    assert_eq!(query.type_schema.as_deref(), Some("person"));
    assert_eq!(query.type_filter, vec!["aut-num", "person"]);
    assert_eq!(
        query.objects,
        vec!["AS4242423011", "MORAXYC-DN42", "-q", "version"]
    );
}

#[test]
fn rejects_missing_values_before_unsupported_options() {
    for (input, expected) in [
        ("-q", "missing value for -q"),
        ("-T", "missing value for -T"),
        ("-t", "missing value for -t"),
        ("-x", "missing value for -x"),
        ("-x nope", "unsupported option -x"),
    ] {
        assert_eq!(Query::parse(input).unwrap_err(), expected, "{input}");
    }
}
