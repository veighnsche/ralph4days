use super::entity::{EntityFile, YamlEntity};
use serde::{Deserialize, Serialize};

/// Discipline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
    #[serde(default)]
    pub acronym: String,
}

// Implement YamlEntity trait for Discipline
impl YamlEntity for Discipline {
    const COLLECTION_NAME: &'static str = "disciplines";
}

/// Manages the disciplines.yaml file
pub type DisciplinesFile = EntityFile<Discipline>;

/// Discipline-specific methods
impl DisciplinesFile {
    /// Load disciplines with default initialization
    pub fn load_with_defaults(&mut self) -> Result<(), String> {
        // If file doesn't exist, initialize with defaults before loading
        if !self.path().exists() {
            self.initialize_defaults();
            return Ok(());
        }
        // Otherwise use standard load
        self.load()
    }

    /// Initialize with 10 default disciplines
    pub fn initialize_defaults(&mut self) {
        *self.items_mut() = vec![
            Discipline {
                name: "frontend".into(),
                display_name: "Frontend".into(),
                acronym: "FRNT".into(),
                icon: "Monitor".into(),
                color: "#3b82f6".into(),
            },
            Discipline {
                name: "backend".into(),
                display_name: "Backend".into(),
                acronym: "BACK".into(),
                icon: "Server".into(),
                color: "#8b5cf6".into(),
            },
            Discipline {
                name: "wiring".into(),
                display_name: "Wiring".into(),
                acronym: "WIRE".into(),
                icon: "Cable".into(),
                color: "#06b6d4".into(),
            },
            Discipline {
                name: "database".into(),
                display_name: "Database".into(),
                acronym: "DTBS".into(),
                icon: "Database".into(),
                color: "#10b981".into(),
            },
            Discipline {
                name: "testing".into(),
                display_name: "Testing".into(),
                acronym: "TEST".into(),
                icon: "FlaskConical".into(),
                color: "#f59e0b".into(),
            },
            Discipline {
                name: "infra".into(),
                display_name: "Infrastructure".into(),
                acronym: "INFR".into(),
                icon: "Cloud".into(),
                color: "#6366f1".into(),
            },
            Discipline {
                name: "security".into(),
                display_name: "Security".into(),
                acronym: "SECR".into(),
                icon: "Shield".into(),
                color: "#ef4444".into(),
            },
            Discipline {
                name: "docs".into(),
                display_name: "Documentation".into(),
                acronym: "DOCS".into(),
                icon: "BookOpen".into(),
                color: "#14b8a6".into(),
            },
            Discipline {
                name: "design".into(),
                display_name: "Design".into(),
                acronym: "DSGN".into(),
                icon: "Palette".into(),
                color: "#ec4899".into(),
            },
            Discipline {
                name: "api".into(),
                display_name: "API".into(),
                acronym: "APIS".into(),
                icon: "Plug".into(),
                color: "#84cc16".into(),
            },
        ];
    }

    /// Validate disciplines collection for uniqueness constraints (called on load)
    pub fn validate(&self) -> Result<(), String> {
        let mut seen_names = std::collections::HashSet::new();
        let mut seen_acronyms = std::collections::HashSet::new();

        for discipline in self.get_all() {
            // Check name uniqueness
            if !seen_names.insert(&discipline.name) {
                return Err(format!(
                    "Duplicate discipline name found: '{}'. Each discipline must have a unique name.",
                    discipline.name
                ));
            }

            // Only validate non-empty acronyms (migration might have empty ones temporarily)
            if !discipline.acronym.is_empty() {
                // Check acronym uniqueness
                if !seen_acronyms.insert(&discipline.acronym) {
                    return Err(format!(
                        "Duplicate discipline acronym found: '{}'. Each discipline must have a unique acronym.",
                        discipline.acronym
                    ));
                }

                // Validate acronym format
                crate::acronym::validate_acronym_format(&discipline.acronym)?;
            }
        }

        Ok(())
    }

    /// Validate that a discipline exists
    pub fn validate_discipline(&self, name: &str) -> Result<(), String> {
        if !self.get_all().iter().any(|d| d.name == name) {
            return Err(format!("Unknown discipline: {}", name));
        }
        Ok(())
    }

    /// Validate acronym is unique within disciplines collection (used during creation)
    fn validate_acronym(&self, acronym: &str, discipline_name: &str) -> Result<(), String> {
        crate::acronym::validate_acronym_format(acronym)?;

        if self
            .get_all()
            .iter()
            .any(|d| d.acronym == acronym && d.name != discipline_name)
        {
            return Err(format!(
                "Acronym '{}' is already used by another discipline. Please choose a unique acronym.",
                acronym
            ));
        }

        Ok(())
    }

}
