use whois42d_ng::types::candidate_types;

#[test]
fn detects_readme_supported_object_types() {
    assert_eq!(candidate_types("AS4242423011"), vec!["aut-num", "as-set"]);
    assert_eq!(candidate_types("moraxyc.dn42"), vec!["dns"]);
    assert_eq!(candidate_types("MORAXYC-MNT"), vec!["mntner"]);
    assert_eq!(candidate_types("RS-DN42-NATIVE"), vec!["route-set"]);
}

#[test]
fn detects_telephony_number() {
    assert_eq!(candidate_types("+04243011"), vec!["telephony"]);
}

#[test]
fn does_not_detect_bare_plus_as_telephony() {
    assert!(candidate_types("+").is_empty());
}

#[test]
fn does_not_detect_bare_as_as_aut_num() {
    assert_eq!(candidate_types("AS"), vec!["as-set"]);
}
