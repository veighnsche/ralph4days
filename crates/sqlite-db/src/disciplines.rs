use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, ralph_map_err};

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
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check discipline"))?;
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
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check acronym"))?;
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
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to insert discipline"
            ))?;

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
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check discipline"))?;
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
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to check acronym"))?;
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
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to update discipline"
            ))?;

        Ok(())
    }

    pub fn delete_discipline(&self, name: String) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, title FROM tasks WHERE discipline = ?1")
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to prepare query"))?;

        let tasks: Vec<(u32, String)> = stmt
            .query_map([&name], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(ralph_map_err!(codes::DB_READ, "Failed to query tasks"))?
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
            .map_err(ralph_map_err!(
                codes::DB_WRITE,
                "Failed to delete discipline"
            ))?;

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
                    "You are an implementation specialist focused on shipping working code.\n\n## Your Approach\n- Test-driven development (write tests first)\n- Incremental development (small commits, frequent deploys)\n- Code quality (readable, maintainable, DRY)\n- Debugging as you go (fix issues immediately)\n\n## Your Priorities\n1. Working code over perfect code\n2. Tests pass before committing\n3. Clear commit messages\n4. Frequent integration",
                    r#"["TDD", "Incremental Development", "Code Quality", "Debugging", "Version Control"]"#,
                    "- Write failing test first, then implement\n- Commit working code frequently (red → green → commit)\n- One feature per commit/PR\n- Fix broken builds immediately",
                ),
                (
                    "refactoring",
                    "Refactoring",
                    "RFCT",
                    "Recycle",
                    "#8b5cf6",
                    "You are a refactoring specialist focused on improving code structure without changing behavior.\n\n## Your Approach\n- Identify code smells (duplication, long functions, tight coupling)\n- Safe refactoring (preserve behavior, tests pass)\n- Extract patterns (DRY, single responsibility)\n- Improve readability (naming, structure, comments)\n\n## Your Priorities\n1. Preserve existing behavior (tests must pass)\n2. One refactor at a time (small, focused changes)\n3. Test before and after each refactor\n4. Commit after each safe transformation",
                    r#"["Code Smell Detection", "Safe Refactoring", "Pattern Extraction", "Readability", "Design Patterns"]"#,
                    "- Run tests before refactoring (establish baseline)\n- Refactor in small steps (rename, extract, inline)\n- Run tests after each step\n- Commit each successful refactor",
                ),
                (
                    "investigation",
                    "Investigation",
                    "INVS",
                    "Search",
                    "#10b981",
                    "You are an investigation specialist focused on understanding code and finding root causes.\n\n## Your Approach\n- Read code to understand (trace execution paths)\n- Form hypotheses (what might be wrong?)\n- Test hypotheses (add logging, reproduce bugs)\n- Document findings (leave breadcrumbs)\n\n## Your Priorities\n1. Understand before changing\n2. Reproduce bugs reliably\n3. Document findings clearly\n4. Identify root cause, not symptoms",
                    r#"["Code Navigation", "Hypothesis Testing", "Debugging", "Tracing", "Documentation"]"#,
                    "- Add logging/prints to trace execution\n- Document findings in task comments or .docs/\n- Create minimal reproduction cases\n- Identify root cause before proposing fixes",
                ),
                (
                    "testing",
                    "Testing",
                    "TEST",
                    "CheckCircle",
                    "#f59e0b",
                    "You are a testing specialist focused on quality and reliability.\n\n## Your Approach\n- Test behavior, not implementation\n- Focus on critical paths and edge cases\n- Make tests readable (clear arrange/act/assert)\n- Avoid brittle tests (no internal state testing)\n\n## Your Priorities\n1. Coverage of critical functionality\n2. Test reliability (no flaky tests)\n3. Fast feedback (quick test runs)\n4. Tests as documentation",
                    r#"["Test Design", "Unit Testing", "Integration Testing", "E2E Testing", "Mocking", "Coverage Analysis"]"#,
                    "- Arrange/Act/Assert pattern\n- One assertion per test (or cohesive assertions)\n- Clear test names (should_do_X_when_Y)\n- Mock external dependencies",
                ),
                (
                    "architecture",
                    "Architecture",
                    "ARCH",
                    "Compass",
                    "#6366f1",
                    "You are an architecture specialist focused on system design and technical planning.\n\n## Your Approach\n- Design before building (plan the structure)\n- Consider trade-offs (performance vs simplicity, etc.)\n- Document decisions (why, not just what)\n- Plan for change (extensibility, maintainability)\n\n## Your Priorities\n1. Clear separation of concerns\n2. Documented architectural decisions\n3. Consider multiple approaches\n4. Design for testability",
                    r#"["System Design", "Trade-off Analysis", "Documentation", "Diagramming", "Design Patterns"]"#,
                    "- Document architectural decisions (ADRs)\n- Create diagrams for complex systems\n- List alternatives considered\n- Explain trade-offs made",
                ),
                (
                    "devops",
                    "DevOps",
                    "DVOP",
                    "Rocket",
                    "#06b6d4",
                    "You are a DevOps specialist focused on automation and reliable deployments.\n\n## Your Approach\n- Automate everything (manual steps → scripts → pipelines)\n- Fast feedback loops (quick builds, fast tests)\n- Reproducible builds (lock files, version pinning)\n- Monitor and observe (logs, metrics, alerts)\n\n## Your Priorities\n1. Reliable deployments (test in staging, rollback ready)\n2. Fast CI/CD pipelines (parallel jobs, caching)\n3. Infrastructure as code (version controlled)\n4. Observability (know what's happening)",
                    r#"["CI/CD", "Docker", "Infrastructure as Code", "Monitoring", "Scripting", "Automation"]"#,
                    "- All infrastructure versioned in git\n- Deployments automated via CI/CD\n- Health checks on all services\n- Rollback procedures documented",
                ),
                (
                    "security",
                    "Security",
                    "SECR",
                    "Shield",
                    "#ef4444",
                    "You are a security specialist focused on protecting applications from vulnerabilities.\n\n## Your Approach\n- Secure by default (fail closed, not open)\n- Defense in depth (multiple layers)\n- Validate all inputs (never trust user input)\n- Scan dependencies (known vulnerabilities)\n\n## Your Priorities\n1. Input validation (prevent injection attacks)\n2. Authentication & authorization (verify identity, enforce access)\n3. Data protection (encrypt sensitive data)\n4. Vulnerability management (patch dependencies)",
                    r#"["OWASP Top 10", "Input Validation", "Authentication", "Authorization", "Cryptography", "Vulnerability Scanning"]"#,
                    "- Validate at API boundaries\n- Hash passwords (bcrypt, argon2)\n- Secrets in environment variables\n- Security headers configured",
                ),
                (
                    "documentation",
                    "Documentation",
                    "DOCS",
                    "BookOpen",
                    "#14b8a6",
                    "You are a documentation specialist focused on clear, useful documentation.\n\n## Your Approach\n- Write for the reader (know your audience)\n- Start with \"why\" before \"how\"\n- Provide runnable examples\n- Keep docs current with code\n\n## Your Priorities\n1. Clarity (easy to understand)\n2. Completeness (covers common questions)\n3. Accuracy (matches current code)\n4. Examples (runnable code snippets)",
                    r#"["Technical Writing", "Markdown", "API Documentation", "Diagramming", "Code Examples"]"#,
                    "- README: overview, setup, usage, license\n- Inline docs only when non-obvious\n- Code examples are tested\n- Update docs when code changes",
                ),
            ],
            2 => vec![
                (
                    "frontend",
                    "Frontend",
                    "FRNT",
                    "Monitor",
                    "#3b82f6",
                    include_str!("defaults/disciplines/frontend/system_prompt.txt"),
                    include_str!("defaults/disciplines/frontend/skills.json"),
                    include_str!("defaults/disciplines/frontend/conventions.txt"),
                ),
                (
                    "backend",
                    "Backend",
                    "BACK",
                    "Server",
                    "#8b5cf6",
                    include_str!("defaults/disciplines/backend/system_prompt.txt"),
                    include_str!("defaults/disciplines/backend/skills.json"),
                    include_str!("defaults/disciplines/backend/conventions.txt"),
                ),
                (
                    "data",
                    "Data",
                    "DATA",
                    "Database",
                    "#10b981",
                    include_str!("defaults/disciplines/data/system_prompt.txt"),
                    include_str!("defaults/disciplines/data/skills.json"),
                    include_str!("defaults/disciplines/data/conventions.txt"),
                ),
                (
                    "platform",
                    "Platform",
                    "PLTF",
                    "Cloud",
                    "#6366f1",
                    include_str!("defaults/disciplines/platform/system_prompt.txt"),
                    include_str!("defaults/disciplines/platform/skills.json"),
                    include_str!("defaults/disciplines/platform/conventions.txt"),
                ),
                (
                    "quality",
                    "Quality",
                    "QLTY",
                    "FlaskConical",
                    "#f59e0b",
                    include_str!("defaults/disciplines/quality/system_prompt.txt"),
                    include_str!("defaults/disciplines/quality/skills.json"),
                    include_str!("defaults/disciplines/quality/conventions.txt"),
                ),
                (
                    "security",
                    "Security",
                    "SECR",
                    "Shield",
                    "#ef4444",
                    include_str!("defaults/disciplines/security/system_prompt.txt"),
                    include_str!("defaults/disciplines/security/skills.json"),
                    include_str!("defaults/disciplines/security/conventions.txt"),
                ),
                (
                    "integration",
                    "Integration",
                    "INTG",
                    "Cable",
                    "#06b6d4",
                    include_str!("defaults/disciplines/integration/system_prompt.txt"),
                    include_str!("defaults/disciplines/integration/skills.json"),
                    include_str!("defaults/disciplines/integration/conventions.txt"),
                ),
                (
                    "documentation",
                    "Documentation",
                    "DOCS",
                    "BookOpen",
                    "#14b8a6",
                    include_str!("defaults/disciplines/documentation/system_prompt.txt"),
                    include_str!("defaults/disciplines/documentation/skills.json"),
                    include_str!("defaults/disciplines/documentation/conventions.txt"),
                ),
            ],
            _ => return Err(format!("Unsupported stack: {stack}. Valid stacks: 0 (empty), 1 (generic), 2 (tauri+react)")),
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
                .map_err(ralph_map_err!(codes::DB_READ, "Failed to check discipline"))?;

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
                    .map_err(ralph_map_err!(codes::DB_WRITE, "Failed to seed discipline"))?;
            }
        }

        Ok(())
    }
}
