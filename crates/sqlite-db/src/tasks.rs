use crate::types::*;
use crate::SqliteDb;
use std::collections::HashSet;

impl SqliteDb {
    /// Create a new task with automatic ID assignment.
    pub fn create_task(&self, input: TaskInput) -> Result<u32, String> {
        if input.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_owned());
        }
        if input.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_owned());
        }
        if input.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_owned());
        }

        // Validate feature exists
        let feature_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&input.feature],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {e}"))?;
        if !feature_exists {
            return Err(format!(
                "Feature '{}' does not exist. Create it first.",
                input.feature
            ));
        }

        // Validate discipline exists
        let discipline_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&input.discipline],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check discipline: {e}"))?;
        if !discipline_exists {
            return Err(format!(
                "Discipline '{}' does not exist. Create it first.",
                input.discipline
            ));
        }

        // Validate dependencies exist
        for dep_id in &input.depends_on {
            let dep_exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                    [dep_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to check dependency: {e}"))?;
            if !dep_exists {
                return Err(format!("Dependency task {dep_id} does not exist"));
            }
        }

        // Assign next ID (MAX(id)+1, matches yaml-db behavior)
        let next_id: u32 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM tasks", [], |row| {
                row.get(0)
            })
            .map_err(|e| format!("Failed to get next task ID: {e}"))?;

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let tags_json = serde_json::to_string(&input.tags).unwrap();
        let depends_on_json = serde_json::to_string(&input.depends_on).unwrap();
        let ac_json =
            serde_json::to_string(&input.acceptance_criteria.unwrap_or_default()).unwrap();
        let cf_json = serde_json::to_string(&input.context_files).unwrap();
        let oa_json = serde_json::to_string(&input.output_artifacts).unwrap();

        self.conn
            .execute(
                "INSERT INTO tasks (id, feature, discipline, title, description, status, priority, \
                 tags, depends_on, created, acceptance_criteria, context_files, output_artifacts, \
                 hints, estimated_turns, provenance) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
                rusqlite::params![
                    next_id,
                    input.feature,
                    input.discipline,
                    input.title,
                    input.description,
                    "pending",
                    input.priority.map(|p| p.as_str().to_owned()),
                    tags_json,
                    depends_on_json,
                    now,
                    ac_json,
                    cf_json,
                    oa_json,
                    input.hints,
                    input.estimated_turns,
                    input.provenance.map(|p| p.as_str().to_owned()),
                ],
            )
            .map_err(|e| format!("Failed to insert task: {e}"))?;

        Ok(next_id)
    }

    /// Update an existing task. Preserves: status, blocked_by, created, completed, provenance, comments.
    pub fn update_task(&self, id: u32, update: TaskInput) -> Result<(), String> {
        if update.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_owned());
        }
        if update.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_owned());
        }
        if update.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_owned());
        }

        // Verify task exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check task: {e}"))?;
        if !exists {
            return Err(format!("Task {id} does not exist"));
        }

        // Validate feature exists
        let feature_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&update.feature],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {e}"))?;
        if !feature_exists {
            return Err(format!(
                "Feature '{}' does not exist. Create it first.",
                update.feature
            ));
        }

        // Validate discipline exists
        let discipline_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM disciplines WHERE name = ?1",
                [&update.discipline],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check discipline: {e}"))?;
        if !discipline_exists {
            return Err(format!(
                "Discipline '{}' does not exist. Create it first.",
                update.discipline
            ));
        }

        // Validate dependencies
        for dep_id in &update.depends_on {
            let dep_exists: bool = self
                .conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                    [dep_id],
                    |row| row.get(0),
                )
                .map_err(|e| format!("Failed to check dependency: {e}"))?;
            if !dep_exists {
                return Err(format!("Dependency task {dep_id} does not exist"));
            }
        }

        // Check self-dependency
        if update.depends_on.contains(&id) {
            return Err(format!("Task {id} cannot depend on itself"));
        }

        // Check circular dependencies
        for dep_id in &update.depends_on {
            if self.has_circular_dependency(id, *dep_id)? {
                return Err(format!(
                    "Circular dependency detected: task {id} would create a cycle with task {dep_id}"
                ));
            }
        }

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();
        let tags_json = serde_json::to_string(&update.tags).unwrap();
        let depends_on_json = serde_json::to_string(&update.depends_on).unwrap();
        let ac_json =
            serde_json::to_string(&update.acceptance_criteria.unwrap_or_default()).unwrap();
        let cf_json = serde_json::to_string(&update.context_files).unwrap();
        let oa_json = serde_json::to_string(&update.output_artifacts).unwrap();

        // Update only mutable columns (preserves status, blocked_by, created, completed, provenance)
        self.conn
            .execute(
                "UPDATE tasks SET feature = ?1, discipline = ?2, title = ?3, description = ?4, \
                 priority = ?5, tags = ?6, depends_on = ?7, updated = ?8, \
                 acceptance_criteria = ?9, context_files = ?10, output_artifacts = ?11, \
                 hints = ?12, estimated_turns = ?13 \
                 WHERE id = ?14",
                rusqlite::params![
                    update.feature,
                    update.discipline,
                    update.title,
                    update.description,
                    update.priority.map(|p| p.as_str().to_owned()),
                    tags_json,
                    depends_on_json,
                    now,
                    ac_json,
                    cf_json,
                    oa_json,
                    update.hints,
                    update.estimated_turns,
                    id,
                ],
            )
            .map_err(|e| format!("Failed to update task: {e}"))?;

        Ok(())
    }

    /// Set a task's status. Sets `completed` timestamp when transitioning to Done.
    pub fn set_task_status(&self, id: u32, status: TaskStatus) -> Result<(), String> {
        // Verify task exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check task: {e}"))?;
        if !exists {
            return Err(format!("Task {id} does not exist"));
        }

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

        if status == TaskStatus::Done {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, completed = ?2, updated = ?3 WHERE id = ?4",
                    rusqlite::params![status.as_str(), now, now, id],
                )
                .map_err(|e| format!("Failed to update task status: {e}"))?;
        } else {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, updated = ?2 WHERE id = ?3",
                    rusqlite::params![status.as_str(), now, id],
                )
                .map_err(|e| format!("Failed to update task status: {e}"))?;
        }

        Ok(())
    }

    /// Delete a task by ID. Fails if other tasks depend on it.
    pub fn delete_task(&self, id: u32) -> Result<(), String> {
        // Check if any tasks depend on this one (app-level check, depends_on is JSON)
        let mut stmt = self
            .conn
            .prepare("SELECT id, depends_on FROM tasks")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let rows = stmt
            .query_map([], |row| {
                let task_id: u32 = row.get(0)?;
                let deps_json: String = row.get(1)?;
                Ok((task_id, deps_json))
            })
            .map_err(|e| format!("Failed to query tasks: {e}"))?;

        for row in rows {
            let (task_id, deps_json) = row.map_err(|e| format!("Failed to read row: {e}"))?;
            let deps: Vec<u32> = serde_json::from_str(&deps_json).unwrap_or_default();
            if deps.contains(&id) {
                return Err(format!(
                    "Cannot delete task {id}: task {task_id} depends on it"
                ));
            }
        }

        // Delete the task (comments cascade via FK)
        let affected = self
            .conn
            .execute("DELETE FROM tasks WHERE id = ?1", [id])
            .map_err(|e| format!("Failed to delete task: {e}"))?;

        if affected == 0 {
            return Err(format!("Task {id} does not exist"));
        }

        Ok(())
    }

    /// Get a task by ID (with joined display fields and inferred status).
    pub fn get_task_by_id(&self, id: u32) -> Option<Task> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.feature, t.discipline, t.title, t.description, t.status, \
                 t.priority, t.tags, t.depends_on, t.blocked_by, t.created, t.updated, \
                 t.completed, t.acceptance_criteria, t.context_files, t.output_artifacts, \
                 t.hints, t.estimated_turns, t.provenance, \
                 COALESCE(f.display_name, t.feature), \
                 COALESCE(f.acronym, t.feature), \
                 COALESCE(d.display_name, t.discipline), \
                 COALESCE(d.acronym, t.discipline), \
                 COALESCE(d.icon, 'Circle'), \
                 COALESCE(d.color, '#94a3b8') \
                 FROM tasks t \
                 LEFT JOIN features f ON t.feature = f.name \
                 LEFT JOIN disciplines d ON t.discipline = d.name \
                 WHERE t.id = ?1",
            )
            .ok()?;

        let mut task = stmt.query_row([id], |row| Ok(self.row_to_task(row))).ok()?;

        task.comments = self.get_comments_for_task(task.id);

        // Compute inferred status using a status map for this task's deps
        let status_map = self.get_task_status_map();
        task.inferred_status =
            Self::compute_inferred_status(task.status, &task.depends_on, &status_map);

        Some(task)
    }

    /// Get all tasks (with joined display fields and inferred status).
    pub fn get_tasks(&self) -> Vec<Task> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT t.id, t.feature, t.discipline, t.title, t.description, t.status, \
                 t.priority, t.tags, t.depends_on, t.blocked_by, t.created, t.updated, \
                 t.completed, t.acceptance_criteria, t.context_files, t.output_artifacts, \
                 t.hints, t.estimated_turns, t.provenance, \
                 COALESCE(f.display_name, t.feature), \
                 COALESCE(f.acronym, t.feature), \
                 COALESCE(d.display_name, t.discipline), \
                 COALESCE(d.acronym, t.discipline), \
                 COALESCE(d.icon, 'Circle'), \
                 COALESCE(d.color, '#94a3b8') \
                 FROM tasks t \
                 LEFT JOIN features f ON t.feature = f.name \
                 LEFT JOIN disciplines d ON t.discipline = d.name \
                 ORDER BY t.id",
            )
            .unwrap();

        let tasks: Vec<Task> = stmt
            .query_map([], |row| Ok(self.row_to_task(row)))
            .unwrap()
            .filter_map(std::result::Result::ok)
            .collect();

        // Build status map for inferred status computation
        let status_map: std::collections::HashMap<u32, TaskStatus> =
            tasks.iter().map(|t| (t.id, t.status)).collect();

        // Load all comments
        let comment_map = self.get_all_comments_by_task();

        tasks
            .into_iter()
            .map(|mut t| {
                t.inferred_status =
                    Self::compute_inferred_status(t.status, &t.depends_on, &status_map);
                t.comments = comment_map.get(&t.id).cloned().unwrap_or_default();
                t
            })
            .collect()
    }

    /// Convert a rusqlite Row to a Task (25 columns: 19 task + 6 joined display).
    /// inferred_status defaults to Ready; caller computes the real value.
    #[allow(clippy::unused_self)]
    fn row_to_task(&self, row: &rusqlite::Row) -> Task {
        let status_str: String = row.get(5).unwrap();
        let priority_str: Option<String> = row.get(6).unwrap();
        let tags_json: String = row.get(7).unwrap();
        let deps_json: String = row.get(8).unwrap();
        let ac_json: String = row.get(13).unwrap();
        let cf_json: String = row.get(14).unwrap();
        let oa_json: String = row.get(15).unwrap();
        let provenance_str: Option<String> = row.get(18).unwrap();

        Task {
            id: row.get(0).unwrap(),
            feature: row.get(1).unwrap(),
            discipline: row.get(2).unwrap(),
            title: row.get(3).unwrap(),
            description: row.get(4).unwrap(),
            status: TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Pending),
            inferred_status: InferredTaskStatus::Ready, // Computed by caller
            priority: priority_str.and_then(|s| Priority::parse(&s)),
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            depends_on: serde_json::from_str(&deps_json).unwrap_or_default(),
            blocked_by: row.get(9).unwrap(),
            created: row.get(10).unwrap(),
            updated: row.get(11).unwrap(),
            completed: row.get(12).unwrap(),
            acceptance_criteria: serde_json::from_str(&ac_json).unwrap_or_default(),
            context_files: serde_json::from_str(&cf_json).unwrap_or_default(),
            output_artifacts: serde_json::from_str(&oa_json).unwrap_or_default(),
            hints: row.get(16).unwrap(),
            estimated_turns: row.get(17).unwrap(),
            provenance: provenance_str.and_then(|s| TaskProvenance::parse(&s)),
            comments: vec![], // Loaded separately
            feature_display_name: row.get(19).unwrap(),
            feature_acronym: row.get(20).unwrap(),
            discipline_display_name: row.get(21).unwrap(),
            discipline_acronym: row.get(22).unwrap(),
            discipline_icon: row.get(23).unwrap(),
            discipline_color: row.get(24).unwrap(),
        }
    }

    /// Build a map of task ID -> TaskStatus for all tasks (used for inferred status).
    fn get_task_status_map(&self) -> std::collections::HashMap<u32, TaskStatus> {
        let mut stmt = self.conn.prepare("SELECT id, status FROM tasks").unwrap();

        stmt.query_map([], |row| {
            let id: u32 = row.get(0)?;
            let status_str: String = row.get(1)?;
            Ok((
                id,
                TaskStatus::parse(&status_str).unwrap_or(TaskStatus::Pending),
            ))
        })
        .unwrap()
        .filter_map(std::result::Result::ok)
        .collect()
    }

    /// Compute inferred status from actual status + dependency graph.
    fn compute_inferred_status(
        status: TaskStatus,
        depends_on: &[u32],
        status_map: &std::collections::HashMap<u32, TaskStatus>,
    ) -> InferredTaskStatus {
        match status {
            TaskStatus::InProgress => InferredTaskStatus::InProgress,
            TaskStatus::Done => InferredTaskStatus::Done,
            TaskStatus::Skipped => InferredTaskStatus::Skipped,
            TaskStatus::Blocked => InferredTaskStatus::ExternallyBlocked,
            TaskStatus::Pending => {
                let all_deps_met = depends_on.iter().all(|dep_id| {
                    status_map
                        .get(dep_id)
                        .is_some_and(|s| *s == TaskStatus::Done)
                });

                if all_deps_met {
                    InferredTaskStatus::Ready
                } else {
                    InferredTaskStatus::WaitingOnDeps
                }
            }
        }
    }

    /// Check if adding task_id -> dep_id would create a circular dependency (DFS).
    fn has_circular_dependency(&self, task_id: u32, dep_id: u32) -> Result<bool, String> {
        // Load all tasks' depends_on
        let mut stmt = self
            .conn
            .prepare("SELECT id, depends_on FROM tasks")
            .map_err(|e| format!("Failed to prepare query: {e}"))?;

        let deps_map: std::collections::HashMap<u32, Vec<u32>> = stmt
            .query_map([], |row| {
                let id: u32 = row.get(0)?;
                let deps_json: String = row.get(1)?;
                let deps: Vec<u32> = serde_json::from_str(&deps_json).unwrap_or_default();
                Ok((id, deps))
            })
            .map_err(|e| format!("Failed to query: {e}"))?
            .filter_map(std::result::Result::ok)
            .collect();

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
