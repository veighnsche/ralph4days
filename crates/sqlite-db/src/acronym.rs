/// Validate acronym format: exactly 4 uppercase alphanumeric characters
pub fn validate_acronym_format(acronym: &str) -> Result<(), String> {
    if acronym.len() != 4 {
        return Err(format!(
            "Acronym must be exactly 4 characters, got: {acronym}"
        ));
    }

    if !acronym
        .chars()
        .all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
    {
        return Err("Acronym must contain only uppercase letters and numbers".to_owned());
    }

    Ok(())
}
