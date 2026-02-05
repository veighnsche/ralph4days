use yaml_db::acronym::{generate_acronym, validate_acronym_format};

#[test]
fn test_validate_format_valid() {
    assert!(validate_acronym_format("FRNT").is_ok());
    assert!(validate_acronym_format("AUTH").is_ok());
    assert!(validate_acronym_format("TEST").is_ok());
    assert!(validate_acronym_format("AB12").is_ok());
}

#[test]
fn test_validate_format_invalid_length() {
    assert!(validate_acronym_format("FRN").is_err()); // too short
    assert!(validate_acronym_format("FRONT").is_err()); // too long
    assert!(validate_acronym_format("").is_err()); // empty
}

#[test]
fn test_validate_format_invalid_chars() {
    assert!(validate_acronym_format("FR_T").is_err()); // underscore
    assert!(validate_acronym_format("frnt").is_err()); // lowercase
    assert!(validate_acronym_format("FR.T").is_err()); // period
    assert!(validate_acronym_format("FR-T").is_err()); // dash
}

#[test]
fn test_generate_single_word() {
    // Single words extract consonants preferentially
    assert_eq!(generate_acronym("frontend", "Frontend"), "FRNT");
    assert_eq!(generate_acronym("backend", "Backend"), "BCKN");
    assert_eq!(generate_acronym("testing", "Testing"), "TSTN");
    assert_eq!(generate_acronym("security", "Security"), "SCRT");
}

#[test]
fn test_generate_multi_word() {
    // Two words: 2 letters from each
    assert_eq!(generate_acronym("user-profile", "User Profile"), "USPR");
    assert_eq!(generate_acronym("api-gateway", "API Gateway"), "APGA");
    assert_eq!(
        generate_acronym("user_authentication", "User Authentication"),
        "USAU"
    );
}

#[test]
fn test_generate_short_names() {
    // Short words repeat last letter to reach 4 chars
    assert_eq!(generate_acronym("api", "API"), "APII"); // repeat I
    assert_eq!(generate_acronym("ui", "UI"), "UIII"); // repeat I
    assert_eq!(generate_acronym("db", "DB"), "DBBB"); // repeat B
}

#[test]
fn test_generate_three_words() {
    assert_eq!(
        generate_acronym("user-access-control", "User Access Control"),
        "USAC"
    );
}

#[test]
fn test_generate_four_plus_words() {
    assert_eq!(
        generate_acronym("very-long-feature-name", "Very Long Feature Name"),
        "VLFN"
    );
    assert_eq!(
        generate_acronym(
            "extremely-verbose-description-here",
            "Extremely Verbose Description Here"
        ),
        "EVDH"
    );
}
