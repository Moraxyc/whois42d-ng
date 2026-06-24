use whois42d_ng::rpsl::RpslObject;

#[test]
fn parses_fields_comments_duplicates_and_continuations() {
    let object = RpslObject::parse(
        r#"% comment
person: MORAXYC-DN42
descr: first
  second
+ third
descr: another
bad line
"#,
    );

    assert_eq!(object.get("PERSON"), Some("MORAXYC-DN42"));
    assert_eq!(object.get("descr"), Some("first\nsecond\nthird"));
    assert_eq!(
        object.get_all("DeScR"),
        vec!["first\nsecond\nthird", "another"]
    );
    assert_eq!(object.fields[0].0, "person");
}
