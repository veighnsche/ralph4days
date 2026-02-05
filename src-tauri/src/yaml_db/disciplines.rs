use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Discipline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discipline {
    pub name: String,
    pub display_name: String,
    pub icon: String,
    pub color: String,
}

/// Manages the disciplines.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisciplinesFile {
    #[serde(skip)]
    path: PathBuf,

    disciplines: Vec<Discipline>,
}

impl DisciplinesFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            disciplines: Vec::new(),
        }
    }

    /// Load disciplines from YAML file
    pub fn load(&mut self) -> Result<(), String> {
        if !self.path.exists() {
            // File doesn't exist yet, initialize with defaults
            self.initialize_defaults();
            return Ok(());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|e| format!("Failed to read disciplines file: {}", e))?;

        let data: DisciplinesData = serde_yaml::from_str(&content)
            .map_err(|e| format!("Failed to parse disciplines YAML: {}", e))?;

        self.disciplines = data.disciplines;
        Ok(())
    }

    /// Save disciplines to YAML file
    pub fn save(&self) -> Result<(), String> {
        let data = DisciplinesData {
            disciplines: self.disciplines.clone(),
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize disciplines: {}", e))?;

        fs::write(&self.path, yaml)
            .map_err(|e| format!("Failed to write disciplines file: {}", e))?;

        Ok(())
    }

    /// Save to temporary file (atomic write pattern - step 1)
    pub fn save_to_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");

        let data = DisciplinesData {
            disciplines: self.disciplines.clone(),
        };

        let yaml = serde_yaml::to_string(&data)
            .map_err(|e| format!("Failed to serialize disciplines: {}", e))?;

        fs::write(&temp_path, yaml)
            .map_err(|e| format!("Failed to write temp disciplines file: {}", e))?;

        Ok(())
    }

    /// Commit temporary file (atomic write pattern - step 2)
    pub fn commit_temp(&self) -> Result<(), String> {
        let temp_path = self.path.with_extension("yaml.tmp");
        fs::rename(&temp_path, &self.path)
            .map_err(|e| format!("Failed to rename temp disciplines file: {}", e))?;
        Ok(())
    }

    /// Rollback temporary file (cleanup on error)
    pub fn rollback_temp(&self) {
        let temp_path = self.path.with_extension("yaml.tmp");
        let _ = fs::remove_file(&temp_path); // Ignore errors
    }

    /// Get all disciplines
    pub fn get_all(&self) -> &[Discipline] {
        &self.disciplines
    }

    /// Initialize with 10 default disciplines
    pub fn initialize_defaults(&mut self) {
        self.disciplines = vec![
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
                color: "#06b6d4".into(),
            },
            Discipline {
                name: "design".into(),
                display_name: "Design".into(),
                icon: "Palette".into(),
                color: "#ec4899".into(),
            },
            Discipline {
                name: "promo".into(),
                display_name: "Promotion".into(),
                icon: "Megaphone".into(),
                color: "#f97316".into(),
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
        if !self.disciplines.iter().any(|d| d.name == name) {
            return Err(format!("Unknown discipline: {}", name));
        }
        Ok(())
    }

    /// Ensure discipline exists (auto-create with generic defaults if not found)
    pub fn ensure_discipline_exists(&mut self, name: &str) -> Result<(), String> {
        if !self.disciplines.iter().any(|d| d.name == name) {
            // Auto-create with generic defaults
            self.disciplines.push(Discipline {
                name: name.to_string(),
                display_name: name.to_string(),
                icon: "Circle".to_string(), // Generic icon
                color: "#94a3b8".to_string(), // Gray default
            });
        }
        Ok(())
    }
}

/// YAML structure for disciplines.yaml file
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DisciplinesData {
    disciplines: Vec<Discipline>,
}
