use super::entity::{EntityFile, YamlEntity};
use serde::{Deserialize, Serialize};

/// Discipline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
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
                icon: "Monitor".into(),
                color: "#3b82f6".into(),
            },
            Discipline {
                name: "backend".into(),
                display_name: "Backend".into(),
                icon: "Server".into(),
                color: "#8b5cf6".into(),
            },
            Discipline {
                name: "wiring".into(),
                display_name: "Wiring".into(),
                icon: "Cable".into(),
                color: "#06b6d4".into(),
            },
            Discipline {
                name: "database".into(),
                display_name: "Database".into(),
                icon: "Database".into(),
                color: "#10b981".into(),
            },
            Discipline {
                name: "testing".into(),
                display_name: "Testing".into(),
                icon: "FlaskConical".into(),
                color: "#f59e0b".into(),
            },
            Discipline {
                name: "infra".into(),
                display_name: "Infrastructure".into(),
                icon: "Cloud".into(),
                color: "#6366f1".into(),
            },
            Discipline {
                name: "security".into(),
                display_name: "Security".into(),
                icon: "Shield".into(),
                color: "#ef4444".into(),
            },
            Discipline {
                name: "docs".into(),
                display_name: "Documentation".into(),
                icon: "BookOpen".into(),
                color: "#14b8a6".into(),
            },
            Discipline {
                name: "design".into(),
                display_name: "Design".into(),
                icon: "Palette".into(),
                color: "#ec4899".into(),
            },
            Discipline {
                name: "api".into(),
                display_name: "API".into(),
                icon: "Plug".into(),
                color: "#84cc16".into(),
            },
        ];
    }

    /// Validate that a discipline exists
    pub fn validate_discipline(&self, name: &str) -> Result<(), String> {
        if !self.get_all().iter().any(|d| d.name == name) {
            return Err(format!("Unknown discipline: {}", name));
        }
        Ok(())
    }

    /// Ensure discipline exists (auto-create with generic defaults if not found)
    pub fn ensure_discipline_exists(&mut self, name: &str) -> Result<(), String> {
        if !self.get_all().iter().any(|d| d.name == name) {
            // Auto-create with generic defaults
            self.add(Discipline {
                name: name.to_string(),
                display_name: name.to_string(),
                icon: "Circle".to_string(),   // Generic icon
                color: "#94a3b8".to_string(), // Gray default
            });
        }
        Ok(())
    }
}
