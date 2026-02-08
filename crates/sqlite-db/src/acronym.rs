use ralph_errors::{codes, ralph_err};

pub fn validate_acronym_format(acronym: &str) -> Result<(), String> {
    if acronym.len() != 4 {
        return ralph_err!(
            codes::TASK_VALIDATION,
            "Acronym must be exactly 4 characters, got: {acronym}"
        );
    }

    if !acronym
        .chars()
        .all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
    {
        return ralph_err!(
            codes::TASK_VALIDATION,
            "Acronym must contain only uppercase letters and numbers"
        );
    }

    Ok(())
}
