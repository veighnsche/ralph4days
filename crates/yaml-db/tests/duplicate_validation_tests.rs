use std::fs;
use tempfile::TempDir;
use yaml_db::YamlDatabase;

fn write_minimal_yaml_files(db_path: &std::path::Path) {
    fs::write(
        db_path.join("metadata.yaml"),
        "schema_version: '1.0'\nproject:\n  title: Test\n",
    )
    .unwrap();
    fs::write(db_path.join("tasks.yaml"), "tasks: []\n").unwrap();
}

#[test]
fn test_load_detects_duplicate_feature_names() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Manually create features.yaml with duplicates
    let features_yaml = concat!(
        "features:\n",
        "- name: auth\n",
        "  display_name: Authentication\n",
        "  acronym: AUTH\n",
        "- name: auth\n",
        "  display_name: Authorization\n",
        "  acronym: AUTZ\n",
    );
    fs::write(db_path.join("features.yaml"), features_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("disciplines.yaml"), "disciplines: []\n").unwrap();

    // Load should fail with clear error
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.contains("Duplicate feature name"), "Error was: {}", err);
        assert!(err.contains("auth"), "Error was: {}", err);
    }
}

#[test]
fn test_load_detects_duplicate_feature_acronyms() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Manually create features.yaml with duplicate acronyms
    let features_yaml = concat!(
        "features:\n",
        "- name: authentication\n",
        "  display_name: Authentication\n",
        "  acronym: AUTH\n",
        "- name: authorization\n",
        "  display_name: Authorization\n",
        "  acronym: AUTH\n",
    );
    fs::write(db_path.join("features.yaml"), features_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("disciplines.yaml"), "disciplines: []\n").unwrap();

    // Load should fail with clear error
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            err.contains("Duplicate feature acronym"),
            "Error was: {}",
            err
        );
        assert!(err.contains("AUTH"), "Error was: {}", err);
    }
}

#[test]
fn test_load_detects_duplicate_discipline_names() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Manually create disciplines.yaml with duplicates
    let disciplines_yaml = concat!(
        "disciplines:\n",
        "- name: frontend\n",
        "  display_name: Frontend\n",
        "  acronym: FRNT\n",
        "  icon: Monitor\n",
        "  color: '#3b82f6'\n",
        "- name: frontend\n",
        "  display_name: Frontend 2\n",
        "  acronym: FNTU\n",
        "  icon: Layout\n",
        "  color: '#3b82f6'\n",
    );
    fs::write(db_path.join("disciplines.yaml"), disciplines_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("features.yaml"), "features: []\n").unwrap();

    // Load should fail with clear error
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            err.contains("Duplicate discipline name"),
            "Error was: {}",
            err
        );
        assert!(err.contains("frontend"), "Error was: {}", err);
    }
}

#[test]
fn test_load_detects_duplicate_discipline_acronyms() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Manually create disciplines.yaml with duplicate acronyms
    let disciplines_yaml = concat!(
        "disciplines:\n",
        "- name: frontend\n",
        "  display_name: Frontend\n",
        "  acronym: FRNT\n",
        "  icon: Monitor\n",
        "  color: '#3b82f6'\n",
        "- name: frontenddev\n",
        "  display_name: Frontend Development\n",
        "  acronym: FRNT\n",
        "  icon: Layout\n",
        "  color: '#3b82f6'\n",
    );
    fs::write(db_path.join("disciplines.yaml"), disciplines_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("features.yaml"), "features: []\n").unwrap();

    // Load should fail with clear error
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            err.contains("Duplicate discipline acronym"),
            "Error was: {}",
            err
        );
        assert!(err.contains("FRNT"), "Error was: {}", err);
    }
}

#[test]
fn test_load_detects_invalid_acronym_format_too_short() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Create features.yaml with invalid acronym (too short)
    let features_yaml = concat!(
        "features:\n",
        "- name: auth\n",
        "  display_name: Auth\n",
        "  acronym: AUT\n",
    );
    fs::write(db_path.join("features.yaml"), features_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("disciplines.yaml"), "disciplines: []\n").unwrap();

    // Load should fail
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            err.contains("Acronym must be exactly 4 characters"),
            "Error was: {}",
            err
        );
    }
}

#[test]
fn test_load_detects_invalid_acronym_format_lowercase() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Create features.yaml with invalid acronym (lowercase)
    let features_yaml = concat!(
        "features:\n",
        "- name: auth\n",
        "  display_name: Auth\n",
        "  acronym: auth\n",
    );
    fs::write(db_path.join("features.yaml"), features_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("disciplines.yaml"), "disciplines: []\n").unwrap();

    // Load should fail
    let result = YamlDatabase::from_path(db_path);
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(
            err.contains("Acronym must contain only uppercase letters and numbers"),
            "Error was: {}",
            err
        );
    }
}

#[test]
fn test_load_succeeds_with_unique_names_and_acronyms() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Create valid YAML files
    let features_yaml = concat!(
        "features:\n",
        "- name: auth\n",
        "  display_name: Authentication\n",
        "  acronym: AUTH\n",
        "- name: profile\n",
        "  display_name: User Profile\n",
        "  acronym: PROF\n",
    );
    fs::write(db_path.join("features.yaml"), features_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("disciplines.yaml"), "disciplines: []\n").unwrap();

    // Load should succeed
    let result = YamlDatabase::from_path(db_path);
    if let Err(err) = &result {
        panic!("Expected success but got error: {}", err);
    }
    assert!(result.is_ok());
}

#[test]
fn test_migration_fills_empty_acronyms() {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("db");
    fs::create_dir_all(&db_path).unwrap();

    // Create features.yaml WITHOUT acronyms (old format)
    let features_yaml = concat!(
        "features:\n",
        "- name: auth\n",
        "  display_name: Authentication\n",
        "- name: profile\n",
        "  display_name: User Profile\n",
    );
    fs::write(db_path.join("features.yaml"), features_yaml).unwrap();
    write_minimal_yaml_files(&db_path);
    fs::write(db_path.join("disciplines.yaml"), "disciplines: []\n").unwrap();

    // Load should succeed and auto-migrate
    let result = YamlDatabase::from_path(db_path.clone());
    if let Err(err) = &result {
        panic!("Expected success but got error: {}", err);
    }

    let db = result.unwrap();
    let features = db.get_features();

    // Verify acronyms were generated
    assert_eq!(features.len(), 2);
    assert!(!features[0].acronym.is_empty());
    assert!(!features[1].acronym.is_empty());
    assert_eq!(features[0].acronym.len(), 4);
    assert_eq!(features[1].acronym.len(), 4);

    // Verify acronyms are different
    assert_ne!(features[0].acronym, features[1].acronym);

    // Reload from disk to verify persistence
    let db2 = YamlDatabase::from_path(db_path).unwrap();
    let features2 = db2.get_features();
    assert_eq!(features2[0].acronym, features[0].acronym);
    assert_eq!(features2[1].acronym, features[1].acronym);
}
