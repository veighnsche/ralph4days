use crate::types::*;
use crate::SqliteDb;
use ralph_errors::{codes, ralph_err, RalphResultExt};
use std::collections::HashSet;

impl SqliteDb {
    #[tracing::instrument(skip(self, input), fields(
        feature = %input.feature,
        discipline = %input.discipline,
        title = %input.title
    ))]
    pub fn create_task(&self, input: TaskInput) -> Result<u32, String> {
        tracing::debug!("Creating task");

        if input.feature.trim().is_empty() {
            tracing::error!("Validation failed: empty feature name");
            return ralph_err!(codes::TASK_VALIDATION, "Feature name cannot be empty");
        }
        if input.discipline.trim().is_empty() {
            tracing::error!("Validation failed: empty discipline name");
            return ralph_err!(codes::TASK_VALIDATION, "Discipline name cannot be empty");
        }
        if input.title.trim().is_empty() {
            tracing::error!("Validation failed: empty task title");
            return ralph_err!(codes::TASK_VALIDATION, "Task title cannot be empty");
        }

        let feature_id = self
            .get_id_from_name("features", &input.feature)
            .map_err(|_| {
                format!(
                    "[R-{}] Feature '{}' does not exist. Create it first.",
                    codes::TASK_VALIDATION,
                    input.feature
                )
            })?;

        let discipline_id = self
            .get_id_from_name("disciplines", &input.discipline)
            .map_err(|_| {
                format!(
                    "[R-{}] Discipline '{}' does not exist. Create it first.",
                    codes::TASK_VALIDATION,
                    input.discipline
                )
            })?;

        for dep_id in &input.depends_on {
            if !self.check_exists("tasks", "id", &dep_id.to_string())? {
                return ralph_err!(
                    codes::TASK_VALIDATION,
                    "Dependency task {dep_id} does not exist"
                );
            }
        }

        let now = self.now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "INSERT INTO tasks (feature_id, discipline_id, title, description, status, priority, \
                 created, hints, estimated_turns, provenance) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                rusqlite::params![
                    feature_id,
                    discipline_id,
                    input.title,
                    input.description,
                    input.status.unwrap_or(TaskStatus::Pending).as_str(),
                    input.priority.map(|p| p.as_str().to_owned()),
                    now,
                    input.hints,
                    input.estimated_turns,
                    input.provenance.map(|p| p.as_str().to_owned()),
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to insert task")?;

        let task_id = self.conn.last_insert_rowid() as u32;

        self.insert_string_list(
            "task_tags",
            "task_id",
            i64::from(task_id),
            "tag",
            &input.tags,
        )?;

        for dep_id in &input.depends_on {
            self.conn
                .execute(
                    "INSERT INTO task_dependencies (task_id, depends_on_task_id) VALUES (?1, ?2)",
                    rusqlite::params![task_id, dep_id],
                )
                .ralph_err(codes::DB_WRITE, "Failed to insert dependency")?;
        }

        let ac = input.acceptance_criteria.unwrap_or_default();
        for (idx, criterion) in ac.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO task_acceptance_criteria (task_id, criterion, criterion_order) VALUES (?1, ?2, ?3)",
                    rusqlite::params![task_id, criterion, i64::try_from(idx).unwrap_or(0)],
                )
                .ralph_err(codes::DB_WRITE, "Failed to insert acceptance criterion")?;
        }

        self.insert_string_list(
            "task_context_files",
            "task_id",
            i64::from(task_id),
            "file_path",
            &input.context_files,
        )?;

        self.insert_string_list(
            "task_output_artifacts",
            "task_id",
            i64::from(task_id),
            "artifact_path",
            &input.output_artifacts,
        )?;

        tracing::info!(
            task_id = task_id,
            feature = %input.feature,
            discipline = %input.discipline,
            "Task created successfully"
        );

        Ok(task_id)
    }

    #[tracing::instrument(skip(self, update), fields(task_id = id))]
    pub fn update_task(&self, id: u32, update: TaskInput) -> Result<(), String> {
        tracing::debug!("Updating task");

        if !self.check_exists("tasks", "id", &id.to_string())? {
            return ralph_err!(codes::TASK_OPS, "Task {id} does not exist");
        }

        let feature_id = self
            .get_id_from_name("features", &update.feature)
            .map_err(|_| {
                format!(
                    "[R-{}] Feature '{}' does not exist. Create it first.",
                    codes::TASK_VALIDATION,
                    update.feature
                )
            })?;

        let discipline_id = self
            .get_id_from_name("disciplines", &update.discipline)
            .map_err(|_| {
                format!(
                    "[R-{}] Discipline '{}' does not exist. Create it first.",
                    codes::TASK_VALIDATION,
                    update.discipline
                )
            })?;

        for dep_id in &update.depends_on {
            if !self.check_exists("tasks", "id", &dep_id.to_string())? {
                return ralph_err!(
                    codes::TASK_VALIDATION,
                    "Dependency task {dep_id} does not exist"
                );
            }
        }

        if update.depends_on.contains(&id) {
            return ralph_err!(codes::TASK_VALIDATION, "Task {id} cannot depend on itself");
        }

        for dep_id in &update.depends_on {
            if self.has_circular_dependency(id, *dep_id)? {
                return ralph_err!(
                    codes::TASK_VALIDATION,
                    "Circular dependency detected: task {id} would create a cycle with task {dep_id}"
                );
            }
        }

        let now = self.now().format("%Y-%m-%d").to_string();

        self.conn
            .execute(
                "UPDATE tasks SET feature_id = ?1, discipline_id = ?2, title = ?3, description = ?4, \
                 priority = ?5, updated = ?6, hints = ?7, estimated_turns = ?8 \
                 WHERE id = ?9",
                rusqlite::params![
                    feature_id,
                    discipline_id,
                    update.title,
                    update.description,
                    update.priority.map(|p| p.as_str().to_owned()),
                    now,
                    update.hints,
                    update.estimated_turns,
                    id,
                ],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update task")?;

        self.conn
            .execute("DELETE FROM task_tags WHERE task_id = ?1", [id])
            .ralph_err(codes::DB_WRITE, "Failed to delete old tags")?;
        self.insert_string_list("task_tags", "task_id", i64::from(id), "tag", &update.tags)?;

        self.conn
            .execute("DELETE FROM task_dependencies WHERE task_id = ?1", [id])
            .ralph_err(codes::DB_WRITE, "Failed to delete old dependencies")?;
        for dep_id in &update.depends_on {
            self.conn
                .execute(
                    "INSERT INTO task_dependencies (task_id, depends_on_task_id) VALUES (?1, ?2)",
                    rusqlite::params![id, dep_id],
                )
                .ralph_err(codes::DB_WRITE, "Failed to insert dependency")?;
        }

        self.conn
            .execute(
                "DELETE FROM task_acceptance_criteria WHERE task_id = ?1",
                [id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete old acceptance criteria")?;
        let ac = update.acceptance_criteria.unwrap_or_default();
        for (idx, criterion) in ac.iter().enumerate() {
            self.conn
                .execute(
                    "INSERT INTO task_acceptance_criteria (task_id, criterion, criterion_order) VALUES (?1, ?2, ?3)",
                    rusqlite::params![id, criterion, i64::try_from(idx).unwrap_or(0)],
                )
                .ralph_err(codes::DB_WRITE, "Failed to insert acceptance criterion")?;
        }

        self.conn
            .execute("DELETE FROM task_context_files WHERE task_id = ?1", [id])
            .ralph_err(codes::DB_WRITE, "Failed to delete old context files")?;
        self.insert_string_list(
            "task_context_files",
            "task_id",
            i64::from(id),
            "file_path",
            &update.context_files,
        )?;

        self.conn
            .execute("DELETE FROM task_output_artifacts WHERE task_id = ?1", [id])
            .ralph_err(codes::DB_WRITE, "Failed to delete old output artifacts")?;
        self.insert_string_list(
            "task_output_artifacts",
            "task_id",
            i64::from(id),
            "artifact_path",
            &update.output_artifacts,
        )?;

        Ok(())
    }

    pub fn set_task_status(&self, id: u32, status: TaskStatus) -> Result<(), String> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !exists {
            return ralph_err!(codes::TASK_OPS, "Task {id} does not exist");
        }

        let now = self.now().format("%Y-%m-%d").to_string();

        if status == TaskStatus::Done {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, completed = ?2, updated = ?3 WHERE id = ?4",
                    rusqlite::params![status.as_str(), now, now, id],
                )
                .ralph_err(codes::DB_WRITE, "Failed to update task status")?;
        } else {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, updated = ?2 WHERE id = ?3",
                    rusqlite::params![status.as_str(), now, id],
                )
                .ralph_err(codes::DB_WRITE, "Failed to update task status")?;
        }

        Ok(())
    }

    /// Set task status with custom timestamp
    ///
    /// **For test fixture generation only.** Production code should use `set_task_status()`.
    pub fn set_task_status_with_date(
        &self,
        id: u32,
        status: TaskStatus,
        date: &str,
    ) -> Result<(), String> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !exists {
            return ralph_err!(codes::TASK_OPS, "Task {id} does not exist");
        }

        if status == TaskStatus::Done {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, completed = ?2, updated = ?3 WHERE id = ?4",
                    rusqlite::params![status.as_str(), date, date, id],
                )
                .ralph_err(codes::DB_WRITE, "Failed to update task status")?;
        } else {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, updated = ?2 WHERE id = ?3",
                    rusqlite::params![status.as_str(), date, id],
                )
                .ralph_err(codes::DB_WRITE, "Failed to update task status")?;
        }

        Ok(())
    }

    /// Set task provenance
    ///
    /// **For test fixture generation only.** Production tasks set provenance at creation.
    pub fn set_task_provenance(&self, id: u32, provenance: TaskProvenance) -> Result<(), String> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .ralph_err(codes::DB_READ, "Failed to check task")?;
        if !exists {
            return ralph_err!(codes::TASK_OPS, "Task {id} does not exist");
        }

        self.conn
            .execute(
                "UPDATE tasks SET provenance = ?1 WHERE id = ?2",
                rusqlite::params![provenance.as_str(), id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to update task provenance")?;

        Ok(())
    }

    pub fn enrich_task(
        &self,
        id: u32,
        pseudocode: &str,
        acceptance_criteria: Option<Vec<String>>,
        context_files: Option<Vec<String>>,
    ) -> Result<(), String> {
        let current_status: String = self
            .conn
            .query_row("SELECT status FROM tasks WHERE id = ?1", [id], |row| {
                row.get(0)
            })
            .ralph_err(codes::DB_READ, "Failed to read task")?;

        if current_status != "draft" {
            return ralph_err!(
                codes::TASK_OPS,
                "Task {id} is not in draft status (current: {current_status})"
            );
        }

        let now = self.now().format("%Y-%m-%d").to_string();
        let ac_json = serde_json::to_string(&acceptance_criteria.unwrap_or_default())
            .ralph_err(codes::DB_WRITE, "Failed to serialize acceptance_criteria")?;
        let cf_json = serde_json::to_string(&context_files.unwrap_or_default())
            .ralph_err(codes::DB_WRITE, "Failed to serialize context_files")?;

        self.conn
            .execute(
                "UPDATE tasks SET pseudocode = ?1, acceptance_criteria = ?2, \
                 context_files = ?3, status = 'pending', enriched_at = ?4, updated = ?5 \
                 WHERE id = ?6",
                rusqlite::params![pseudocode, ac_json, cf_json, now, now, id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to enrich task")?;

        Ok(())
    }

    pub fn delete_task(&self, id: u32) -> Result<(), String> {
        let mut stmt = self
            .conn
            .prepare("SELECT task_id FROM task_dependencies WHERE depends_on_task_id = ?1 LIMIT 1")
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        if let Ok(task_id) = stmt.query_row([id], |row| row.get::<_, u32>(0)) {
            return ralph_err!(
                codes::TASK_OPS,
                "Cannot delete task {id}: task {task_id} depends on it"
            );
        }

        let affected = self
            .conn
            .execute("DELETE FROM tasks WHERE id = ?1", [id])
            .ralph_err(codes::DB_WRITE, "Failed to delete task")?;

        if affected == 0 {
            return ralph_err!(codes::TASK_OPS, "Task {id} does not exist");
        }

        Ok(())
    }

    pub fn get_task_by_id(&self, id: u32) -> Option<Task> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.feature_id, t.discipline_id, t.title, t.description, t.status, \
                 t.priority, t.created, t.updated, t.completed, t.hints, t.estimated_turns, \
                 t.provenance, t.pseudocode, t.enriched_at, \
                 f.name, f.display_name, f.acronym, \
                 d.name, d.display_name, d.acronym, d.icon, d.color \
                 FROM tasks t \
                 JOIN features f ON t.feature_id = f.id \
                 JOIN disciplines d ON t.discipline_id = d.id \
                 WHERE t.id = ?1",
            )
            .ok()?;

        let mut task = stmt.query_row([id], |row| Ok(self.row_to_task(row))).ok()?;

        task.tags = self.read_string_list("task_tags", "task_id", i64::from(task.id), "tag");
        task.depends_on = self.read_task_dependencies(task.id);
        task.acceptance_criteria = self.read_acceptance_criteria(task.id);
        task.context_files = self.read_string_list(
            "task_context_files",
            "task_id",
            i64::from(task.id),
            "file_path",
        );
        task.output_artifacts = self.read_string_list(
            "task_output_artifacts",
            "task_id",
            i64::from(task.id),
            "artifact_path",
        );
        task.signals = self.get_signals_for_task(task.id);

        Some(task)
    }

    pub fn get_tasks(&self) -> Vec<Task> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT t.id, t.feature_id, t.discipline_id, t.title, t.description, t.status, \
             t.priority, t.created, t.updated, t.completed, t.hints, t.estimated_turns, \
             t.provenance, t.pseudocode, t.enriched_at, \
             f.name, f.display_name, f.acronym, \
             d.name, d.display_name, d.acronym, d.icon, d.color \
             FROM tasks t \
             JOIN features f ON t.feature_id = f.id \
             JOIN disciplines d ON t.discipline_id = d.id \
             ORDER BY t.id",
        ) else {
            return vec![];
        };

        let Ok(rows) = stmt.query_map([], |row| Ok(self.row_to_task(row))) else {
            return vec![];
        };

        let mut tasks: Vec<Task> = rows.filter_map(std::result::Result::ok).collect();

        for task in &mut tasks {
            task.tags = self.read_string_list("task_tags", "task_id", i64::from(task.id), "tag");
            task.depends_on = self.read_task_dependencies(task.id);
            task.acceptance_criteria = self.read_acceptance_criteria(task.id);
            task.context_files = self.read_string_list(
                "task_context_files",
                "task_id",
                i64::from(task.id),
                "file_path",
            );
            task.output_artifacts = self.read_string_list(
                "task_output_artifacts",
                "task_id",
                i64::from(task.id),
                "artifact_path",
            );
        }

        let comment_map = self.get_all_signals_by_task();

        tasks
            .into_iter()
            .map(|mut t| {
                t.signals = comment_map.get(&t.id).cloned().unwrap_or_default();
                t
            })
            .collect()
    }

    #[allow(clippy::unused_self)]
    fn row_to_task(&self, row: &rusqlite::Row) -> Task {
        let status_str: String = row.get(5).unwrap_or_else(|_| "pending".to_owned());
        let priority_str: Option<String> = row.get(6).ok();
        let provenance_str: Option<String> = row.get(12).ok();

        Task {
            id: row.get(0).unwrap_or(0),
            feature: row.get(15).unwrap_or_default(),
            discipline: row.get(18).unwrap_or_default(),
            title: row.get(3).unwrap_or_default(),
            description: row.get(4).unwrap_or_default(),
            status: TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Pending),
            priority: priority_str.and_then(|s| Priority::parse(&s)),
            tags: vec![],
            depends_on: vec![],
            created: row.get(7).unwrap_or_default(),
            updated: row.get(8).ok(),
            completed: row.get(9).ok(),
            acceptance_criteria: vec![],
            context_files: vec![],
            output_artifacts: vec![],
            hints: row.get(10).ok(),
            estimated_turns: row.get(11).ok(),
            provenance: provenance_str.and_then(|s| TaskProvenance::parse(&s)),
            pseudocode: row.get(13).ok(),
            enriched_at: row.get(14).ok(),
            signals: vec![],
            feature_display_name: row.get(16).unwrap_or_default(),
            feature_acronym: row.get(17).unwrap_or_default(),
            discipline_display_name: row.get(19).unwrap_or_default(),
            discipline_acronym: row.get(20).unwrap_or_default(),
            discipline_icon: row.get(21).unwrap_or_else(|_| "Circle".to_owned()),
            discipline_color: row.get(22).unwrap_or_else(|_| "#94a3b8".to_owned()),
        }
    }

    fn read_task_dependencies(&self, task_id: u32) -> Vec<u32> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT depends_on_task_id FROM task_dependencies WHERE task_id = ? ORDER BY id",
        ) else {
            return vec![];
        };

        let Ok(rows) = stmt.query_map([task_id], |row| row.get::<_, u32>(0)) else {
            return vec![];
        };

        rows.filter_map(std::result::Result::ok).collect()
    }

    fn read_acceptance_criteria(&self, task_id: u32) -> Vec<String> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT criterion FROM task_acceptance_criteria WHERE task_id = ? ORDER BY criterion_order",
        ) else {
            return vec![];
        };

        let Ok(rows) = stmt.query_map([task_id], |row| row.get::<_, String>(0)) else {
            return vec![];
        };

        rows.filter_map(std::result::Result::ok).collect()
    }

    fn has_circular_dependency(&self, task_id: u32, dep_id: u32) -> Result<bool, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT task_id, depends_on_task_id FROM task_dependencies")
            .ralph_err(codes::DB_READ, "Failed to prepare query")?;

        let mut deps_map: std::collections::HashMap<u32, Vec<u32>> =
            std::collections::HashMap::new();
        let Ok(rows) = stmt.query_map([], |row| Ok((row.get::<_, u32>(0)?, row.get::<_, u32>(1)?)))
        else {
            return Ok(false);
        };

        for row in rows.filter_map(std::result::Result::ok) {
            deps_map.entry(row.0).or_default().push(row.1);
        }

        let mut visited = HashSet::new();
        let mut stack = vec![dep_id];

        while let Some(current_id) = stack.pop() {
            if current_id == task_id {
                return Ok(true);
            }
            if !visited.insert(current_id) {
                continue;
            }
            if let Some(deps) = deps_map.get(&current_id) {
                for &next_dep in deps {
                    stack.push(next_dep);
                }
            }
        }

        Ok(false)
    }
}
