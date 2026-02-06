use yaml_db::acronym::validate_acronym_format;

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
