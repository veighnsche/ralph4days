use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
struct StackAbout {
    stack_id: u8,
    name: String,
    description: String,
    philosophy: String,
    when_to_use: Vec<String>,
    approach: String,
    discipline_count: u8,
    characteristics: Vec<String>,
}

fn load_about(stack_path: &str) -> Result<StackAbout, String> {
    let path = Path::new(stack_path).join("ABOUT.yaml");
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse YAML in {}: {}", path.display(), e))
}

#[test]
fn test_all_stacks_have_about_yaml() {
    let stacks = [
        "src/defaults/disciplines/01_generic",
        "src/defaults/disciplines/02_desktop",
        "src/defaults/disciplines/03_saas",
        "src/defaults/disciplines/04_mobile",
    ];

    for stack_path in stacks {
        let about_path = Path::new(stack_path).join("ABOUT.yaml");
        assert!(about_path.exists(), "Missing ABOUT.yaml in {stack_path}");
    }
}

#[test]
fn test_about_yaml_schema_validation() {
    let stacks = [
        ("src/defaults/disciplines/01_generic", 1, "Generic", 8),
        ("src/defaults/disciplines/02_desktop", 2, "Desktop", 8),
        ("src/defaults/disciplines/03_saas", 3, "SaaS", 8),
        ("src/defaults/disciplines/04_mobile", 4, "Mobile", 8),
    ];

    for (stack_path, expected_id, expected_name_contains, expected_disc_count) in stacks {
        let about = load_about(stack_path)
            .unwrap_or_else(|e| panic!("Failed to load ABOUT.yaml from {stack_path}: {e}"));

        assert_eq!(
            about.stack_id, expected_id,
            "{stack_path}: stack_id should be {expected_id}"
        );

        assert!(
            about.name.contains(expected_name_contains),
            "{stack_path}: name '{}' should contain '{expected_name_contains}'",
            about.name
        );

        assert!(
            !about.description.is_empty(),
            "{stack_path}: description cannot be empty"
        );
        assert!(
            !about.philosophy.is_empty(),
            "{stack_path}: philosophy cannot be empty"
        );

        assert!(
            !about.when_to_use.is_empty(),
            "{stack_path}: when_to_use must have at least one item"
        );

        assert!(
            matches!(about.approach.as_str(), "mode-based" | "tech-specific"),
            "{stack_path}: approach must be 'mode-based' or 'tech-specific', got '{}'",
            about.approach
        );

        assert_eq!(
            about.discipline_count, expected_disc_count,
            "{stack_path}: discipline_count should be {expected_disc_count}"
        );

        assert!(
            !about.characteristics.is_empty(),
            "{stack_path}: characteristics must have at least one item"
        );
    }
}

#[test]
fn test_stack_0_empty_has_no_about_yaml() {
    let empty_stack_path = "src/defaults/disciplines/00_empty";

    if Path::new(empty_stack_path).exists() {
        let about_path = Path::new(empty_stack_path).join("ABOUT.yaml");
        assert!(
            !about_path.exists(),
            "Stack 0 (empty) should NOT have ABOUT.yaml - it has zero disciplines"
        );
    }
}

#[test]
fn test_approach_consistency() {
    let generic = load_about("src/defaults/disciplines/01_generic")
        .expect("Failed to load 01_generic/ABOUT.yaml");

    assert_eq!(
        generic.approach, "mode-based",
        "Stack 1 (Generic) must be mode-based"
    );

    let desktop = load_about("src/defaults/disciplines/02_desktop")
        .expect("Failed to load 02_desktop/ABOUT.yaml");

    assert_eq!(
        desktop.approach, "tech-specific",
        "Stack 2 (Desktop) must be tech-specific"
    );
}
