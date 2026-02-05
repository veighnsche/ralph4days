//! Migration script to update transformation snapshots with correct YAML formatting
//! Run with: cargo test --test migrate_snapshots -- --nocapture

use std::fs;
use std::path::PathBuf;
use yaml_db::YamlDatabase;

#[test]
fn migrate_all_transformation_snapshots() {
    let base = PathBuf::from("tests/snapshots/transformations");

    for entry in fs::read_dir(&base).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let test_name = entry.file_name();
        println!("Processing {:?}...", test_name);

        // Process both before and after directories
        for subdir in &["before", "after"] {
            let snapshot_dir = entry.path().join(subdir);
            if !snapshot_dir.exists() {
                continue;
            }

            // Check if it's a valid database directory
            let disciplines_file = snapshot_dir.join("disciplines.yaml");
            if disciplines_file.exists() {
                // Load the database (this will auto-migrate acronyms if missing)
                match YamlDatabase::from_path(snapshot_dir.clone()) {
                    Ok(db) => {
                        // Save all files with correct field ordering
                        db.save_all().unwrap();
                        println!("  ✓ Migrated {}/{:?}", subdir, test_name);
                    }
                    Err(e) => {
                        println!("  ✗ Failed {}/{:?}: {}", subdir, test_name, e);
                    }
                }
            }
        }
    }

    println!("\n✅ All snapshots migrated with correct field ordering!");
}
