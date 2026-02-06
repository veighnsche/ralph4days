use crate::YamlDatabase;
use std::collections::HashSet;

/// Simple acronym generation for migration only: uppercase, strip separators, take first 4 chars, pad if short.
fn simple_acronym(name: &str) -> String {
    let raw: String = name
        .to_uppercase()
        .replace('-', "")
        .replace('_', "")
        .chars()
        .take(4)
        .collect();
    if raw.len() >= 4 {
        raw
    } else {
        // Pad with last char to reach 4
        let last = raw.chars().last().unwrap_or('X');
        format!("{}{}", raw, last.to_string().repeat(4 - raw.len()))
    }
}

/// Migrate existing features/disciplines to include acronyms
/// This runs once when loading old projects without acronyms
pub fn migrate_acronyms_if_needed(db: &mut YamlDatabase) -> Result<(), String> {
    let mut needs_save = false;
    let mut used_acronyms = HashSet::new();

    // Migrate features
    for feature in db.features.items_mut() {
        if feature.acronym.is_empty() {
            let generated = simple_acronym(&feature.name);

            // Handle collisions by appending numbers
            let mut acronym = generated.clone();
            let mut counter = 1;
            while used_acronyms.contains(&acronym) {
                acronym = format!("{}{:03}", &generated[..1], counter);
                counter += 1;
            }

            feature.acronym = acronym.clone();
            used_acronyms.insert(acronym);
            needs_save = true;
        } else {
            used_acronyms.insert(feature.acronym.clone());
        }
    }

    // Migrate disciplines (same pattern)
    used_acronyms.clear();
    for discipline in db.disciplines.items_mut() {
        if discipline.acronym.is_empty() {
            let generated = simple_acronym(&discipline.name);

            let mut acronym = generated.clone();
            let mut counter = 1;
            while used_acronyms.contains(&acronym) {
                acronym = format!("{}{:03}", &generated[..1], counter);
                counter += 1;
            }

            discipline.acronym = acronym.clone();
            used_acronyms.insert(acronym);
            needs_save = true;
        } else {
            used_acronyms.insert(discipline.acronym.clone());
        }
    }

    if needs_save {
        db.save_all()?;
    }

    Ok(())
}
