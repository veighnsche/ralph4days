use crate::{acronym::generate_acronym, YamlDatabase};
use std::collections::HashSet;

/// Migrate existing features/disciplines to include acronyms
/// This runs once when loading old projects without acronyms
pub fn migrate_acronyms_if_needed(db: &mut YamlDatabase) -> Result<(), String> {
    let mut needs_save = false;
    let mut used_acronyms = HashSet::new();

    // Migrate features
    for feature in db.features.items_mut() {
        if feature.acronym.is_empty() {
            let generated = generate_acronym(&feature.name, &feature.display_name);

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
            let generated = generate_acronym(&discipline.name, &discipline.display_name);

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
