/// Validate acronym format: exactly 4 uppercase alphanumeric characters
pub fn validate_acronym_format(acronym: &str) -> Result<(), String> {
    if acronym.len() != 4 {
        return Err(format!(
            "Acronym must be exactly 4 characters, got: {}",
            acronym
        ));
    }

    if !acronym
        .chars()
        .all(|c| c.is_ascii_alphanumeric() && (c.is_ascii_uppercase() || c.is_ascii_digit()))
    {
        return Err("Acronym must contain only uppercase letters and numbers".to_string());
    }

    Ok(())
}

/// Generate 4-letter acronym from name (used only during migration)
/// Examples: "frontend" → "FRNT", "user-profile" → "USPR", "api" → "APIS"
pub fn generate_acronym(name: &str, display_name: &str) -> String {
    let source = if display_name.contains(' ') {
        display_name
    } else {
        name
    };

    let cleaned = source.to_uppercase().replace('-', " ").replace('_', " ");
    let words: Vec<&str> = cleaned.split_whitespace().collect();

    let acronym = match words.len() {
        0 => "UNKN".to_string(),
        1 => {
            let word = words[0];
            if word.len() >= 4 {
                // Extract consonants preferentially, fall back to first 4
                let consonants: String = word
                    .chars()
                    .filter(|c| !"AEIOU".contains(*c))
                    .take(4)
                    .collect();

                if consonants.len() >= 4 {
                    consonants
                } else {
                    word.chars().take(4).collect()
                }
            } else {
                // Pad short words by repeating last letter
                if let Some(last_char) = word.chars().last() {
                    format!(
                        "{}{}",
                        word,
                        last_char.to_string().repeat(4 - word.len())
                    )
                } else {
                    "UNKN".to_string()
                }
            }
        }
        2 => {
            // 2 letters from each word
            format!(
                "{}{}",
                words[0].chars().take(2).collect::<String>(),
                words[1].chars().take(2).collect::<String>()
            )
        }
        3 => {
            // 2 from first, 1 from second, 1 from third
            format!(
                "{}{}{}",
                words[0].chars().take(2).collect::<String>(),
                words[1].chars().next().unwrap_or('X'),
                words[2].chars().next().unwrap_or('X')
            )
        }
        _ => {
            // 4+ words: first letter from first 4 words
            words
                .iter()
                .take(4)
                .filter_map(|w| w.chars().next())
                .collect()
        }
    };

    acronym.to_uppercase()
}
