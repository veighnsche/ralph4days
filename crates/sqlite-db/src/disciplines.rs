use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};

impl SqliteDb {
    pub fn create_discipline(&self, input: crate::types::DisciplineInput) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline name cannot be empty");
        }
        if input.display_name.trim().is_empty() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline display name cannot be empty"
            );
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check discipline")?;
        if exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline '{}' already exists",
                input.name
            );
        }

        let acronym_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1",
                [&input.acronym],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check acronym")?;
        if acronym_exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{}' is already used by another discipline",
                input.acronym
            );
        }

        self.conn
            .execute(
                "INSERT INTO disciplines (name, display_name, acronym, icon, color, \
                 system_prompt, skills, conventions, mcp_servers) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    input.name,
                    input.display_name,
                    input.acronym,
                    input.icon,
                    input.color,
                    input.system_prompt,
                    input.skills,
                    input.conventions,
                    input.mcp_servers
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert discipline")?;

        Ok(())
    }

    pub fn update_discipline(&self, input: crate::types::DisciplineInput) -> Result<(), String> {
        if input.display_name.trim().is_empty() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline display name cannot be empty"
            );
        }

        crate::acronym::validate_acronym_format(&input.acronym)?;

        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check discipline")?;
        if !exists {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Discipline '{}' does not exist",
                input.name
            );
        }

        let acronym_conflict: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE acronym = ?1 AND name != ?2",
                rusqlite::params![input.acronym, input.name],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check acronym")?;
        if acronym_conflict {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Acronym '{}' is already used by another discipline",
                input.acronym
            );
        }

        self.conn
            .execute(
                "UPDATE disciplines SET display_name = ?1, acronym = ?2, icon = ?3, color = ?4, \
                 system_prompt = ?5, skills = ?6, conventions = ?7, mcp_servers = ?8 \
                 WHERE name = ?9",
                rusqlite::params![
                    input.display_name,
                    input.acronym,
                    input.icon,
                    input.color,
                    input.system_prompt,
                    input.skills,
                    input.conventions,
                    input.mcp_servers,
                    input.name
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update discipline")?;

        Ok(())
    }

    pub fn delete_discipline(&self, name: String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE discipline = ?1")
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .ralph_err(codes::DB_READ, "Failed to query tasks")?
            .filter_map(std::result::Result::ok)
            .collect();

        if let Some((task_id, task_title)) = tasks.first() {
            return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Cannot delete discipline '{name}': task {task_id} ('{task_title}') belongs to it"
            );
        }

        let affected = self
            .conn
            .execute("DELETE FROM disciplines WHERE name = ?1", [&name])
            .ralph_err(codes::DB_WRITE, "Failed to delete discipline")?;

        if affected == 0 {
            return ralph_err!(codes::DISCIPLINE_OPS, "Discipline '{name}' does not exist");
        }

        Ok(())
    }

    pub fn get_disciplines(&self) -> Vec<Discipline> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT name, display_name, acronym, icon, color, system_prompt, skills, \
             conventions, mcp_servers \
             FROM disciplines ORDER BY name",
        ) else {
            return vec![];
        };

        stmt.query_map([], |row| {
            let skills_json: String = row.get(6)?;
            let mcp_json: String = row.get(8)?;
            Ok(Discipline {
                name: row.get(0)?,
                display_name: row.get(1)?,
                acronym: row.get(2)?,
                icon: row.get(3)?,
                color: row.get(4)?,
                system_prompt: row.get(5)?,
                skills: serde_json::from_str(&skills_json).unwrap_or_default(),
                conventions: row.get(7)?,
                mcp_servers: serde_json::from_str(&mcp_json).unwrap_or_default(),
            })
        })
        .map_or_else(
            |_| vec![],
            |rows| rows.filter_map(std::result::Result::ok).collect(),
        )
    }

    pub fn seed_defaults(&self) -> Result<(), String> {
        self.seed_for_stack(2)
    }

    pub fn seed_for_stack(&self, stack: u8) -> Result<(), String> {
        type DisciplineSeed = (
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
            &'static str,
        );

        let defaults: Vec<DisciplineSeed> = match stack {
            0 => vec![],
            1 => vec![
                (
                    "implementation",
                    "Implementation",
                    "IMPL",
                    "Hammer",
                    "#3b82f6",
                    include_str!("defaults/disciplines/stack1/implementation/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/implementation/skills.json"),
                    include_str!("defaults/disciplines/stack1/implementation/conventions.txt"),
                ),
                (
                    "refactoring",
                    "Refactoring",
                    "RFCT",
                    "Recycle",
                    "#8b5cf6",
                    include_str!("defaults/disciplines/stack1/refactoring/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/refactoring/skills.json"),
                    include_str!("defaults/disciplines/stack1/refactoring/conventions.txt"),
                ),
                (
                    "investigation",
                    "Investigation",
                    "INVS",
                    "Search",
                    "#10b981",
                    include_str!("defaults/disciplines/stack1/investigation/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/investigation/skills.json"),
                    include_str!("defaults/disciplines/stack1/investigation/conventions.txt"),
                ),
                (
                    "testing",
                    "Testing",
                    "TEST",
                    "CheckCircle",
                    "#f59e0b",
                    include_str!("defaults/disciplines/stack1/testing/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/testing/skills.json"),
                    include_str!("defaults/disciplines/stack1/testing/conventions.txt"),
                ),
                (
                    "architecture",
                    "Architecture",
                    "ARCH",
                    "Compass",
                    "#6366f1",
                    include_str!("defaults/disciplines/stack1/architecture/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/architecture/skills.json"),
                    include_str!("defaults/disciplines/stack1/architecture/conventions.txt"),
                ),
                (
                    "devops",
                    "DevOps",
                    "DVOP",
                    "Rocket",
                    "#06b6d4",
                    include_str!("defaults/disciplines/stack1/devops/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/devops/skills.json"),
                    include_str!("defaults/disciplines/stack1/devops/conventions.txt"),
                ),
                (
                    "security",
                    "Security",
                    "SECR",
                    "Shield",
                    "#ef4444",
                    include_str!("defaults/disciplines/stack1/security/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/security/skills.json"),
                    include_str!("defaults/disciplines/stack1/security/conventions.txt"),
                ),
                (
                    "documentation",
                    "Documentation",
                    "DOCS",
                    "BookOpen",
                    "#14b8a6",
                    include_str!("defaults/disciplines/stack1/documentation/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack1/documentation/skills.json"),
                    include_str!("defaults/disciplines/stack1/documentation/conventions.txt"),
                ),
            ],
            2 => vec![
                (
                    "frontend",
                    "Frontend",
                    "FRNT",
                    "Monitor",
                    "#3b82f6",
                    include_str!("defaults/disciplines/stack2/frontend/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/frontend/skills.json"),
                    include_str!("defaults/disciplines/stack2/frontend/conventions.txt"),
                ),
                (
                    "backend",
                    "Backend",
                    "BACK",
                    "Server",
                    "#8b5cf6",
                    include_str!("defaults/disciplines/stack2/backend/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/backend/skills.json"),
                    include_str!("defaults/disciplines/stack2/backend/conventions.txt"),
                ),
                (
                    "data",
                    "Data",
                    "DATA",
                    "Database",
                    "#10b981",
                    include_str!("defaults/disciplines/stack2/data/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/data/skills.json"),
                    include_str!("defaults/disciplines/stack2/data/conventions.txt"),
                ),
                (
                    "platform",
                    "Platform",
                    "PLTF",
                    "Cloud",
                    "#6366f1",
                    include_str!("defaults/disciplines/stack2/platform/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/platform/skills.json"),
                    include_str!("defaults/disciplines/stack2/platform/conventions.txt"),
                ),
                (
                    "quality",
                    "Quality",
                    "QLTY",
                    "FlaskConical",
                    "#f59e0b",
                    include_str!("defaults/disciplines/stack2/quality/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/quality/skills.json"),
                    include_str!("defaults/disciplines/stack2/quality/conventions.txt"),
                ),
                (
                    "security",
                    "Security",
                    "SECR",
                    "Shield",
                    "#ef4444",
                    include_str!("defaults/disciplines/stack2/security/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/security/skills.json"),
                    include_str!("defaults/disciplines/stack2/security/conventions.txt"),
                ),
                (
                    "integration",
                    "Integration",
                    "INTG",
                    "Cable",
                    "#06b6d4",
                    include_str!("defaults/disciplines/stack2/integration/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/integration/skills.json"),
                    include_str!("defaults/disciplines/stack2/integration/conventions.txt"),
                ),
                (
                    "documentation",
                    "Documentation",
                    "DOCS",
                    "BookOpen",
                    "#14b8a6",
                    include_str!("defaults/disciplines/stack2/documentation/system_prompt.txt"),
                    include_str!("defaults/disciplines/stack2/documentation/skills.json"),
                    include_str!("defaults/disciplines/stack2/documentation/conventions.txt"),
                ),
            ],
            _ => {
                return ralph_err!(
                codes::DISCIPLINE_OPS,
                "Unsupported stack: {stack}. Valid stacks: 0 (empty), 1 (generic), 2 (tauri+react)"
            )
            }
        };

        for (name, display_name, acronym, icon, color, system_prompt, skills, conventions) in
            defaults
        {
            let exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                    [name],
                    |row| row.get(0),
                )
                .ralph_err(codes::DB_READ, "Failed to check discipline")?;

            if !exists {
                self.conn
                    .execute(
                        "INSERT INTO disciplines (name, display_name, acronym, icon, color, \
                         system_prompt, skills, conventions) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        rusqlite::params![
                            name,
                            display_name,
                            acronym,
                            icon,
                            color,
                            system_prompt,
                            skills,
                            conventions
                        ],
                    )
                    .ralph_err(codes::DB_WRITE, "Failed to seed discipline")?;
            }
        }

        Ok(())
    }
}
