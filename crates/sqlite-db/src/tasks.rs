use crate::types::*;
use crate::SqliteDb;
use std::collections::HashSet;

impl SqliteDb {
    /// Create a new task with automatic ID assignment.
    pub fn create_task(&self, input: TaskInput) -> Result<u32, String> {
        if input.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if input.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if input.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        // Validate feature exists
        let feature_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&input.feature],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {}", e))?;
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
            .map_err(|e| format!("Failed to check discipline: {}", e))?;
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
                .map_err(|e| format!("Failed to check dependency: {}", e))?;
            if !dep_exists {
                return Err(format!("Dependency task {} does not exist", dep_id));
            }
        }

        // Assign next ID (MAX(id)+1, matches yaml-db behavior)
        let next_id: u32 = self
            .conn
            .query_row("SELECT COALESCE(MAX(id), 0) + 1 FROM tasks", [], |row| {
                row.get(0)
            })
            .map_err(|e| format!("Failed to get next task ID: {}", e))?;

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
                    input.priority.map(|p| p.as_str().to_string()),
                    tags_json,
                    depends_on_json,
                    now,
                    ac_json,
                    cf_json,
                    oa_json,
                    input.hints,
                    input.estimated_turns,
                    input.provenance.map(|p| p.as_str().to_string()),
                ],
            )
            .map_err(|e| format!("Failed to insert task: {}", e))?;

        Ok(next_id)
    }

    /// Update an existing task. Preserves: status, blocked_by, created, completed, provenance, comments.
    pub fn update_task(&self, id: u32, update: TaskInput) -> Result<(), String> {
        if update.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if update.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if update.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        // Verify task exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM tasks WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check task: {}", e))?;
        if !exists {
            return Err(format!("Task {} does not exist", id));
        }

        // Validate feature exists
        let feature_exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM features WHERE name = ?1",
                [&update.feature],
                |row| row.get(0),
            )
            .map_err(|e| format!("Failed to check feature: {}", e))?;
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
            .map_err(|e| format!("Failed to check discipline: {}", e))?;
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
                .map_err(|e| format!("Failed to check dependency: {}", e))?;
            if !dep_exists {
                return Err(format!("Dependency task {} does not exist", dep_id));
            }
        }

        // Check self-dependency
        if update.depends_on.contains(&id) {
            return Err(format!("Task {} cannot depend on itself", id));
        }

        // Check circular dependencies
        for dep_id in &update.depends_on {
            if self.has_circular_dependency(id, *dep_id)? {
                return Err(format!(
                    "Circular dependency detected: task {} would create a cycle with task {}",
                    id, dep_id
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
                    update.priority.map(|p| p.as_str().to_string()),
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
            .map_err(|e| format!("Failed to update task: {}", e))?;

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
            .map_err(|e| format!("Failed to check task: {}", e))?;
        if !exists {
            return Err(format!("Task {} does not exist", id));
        }

        let now = chrono::Utc::now().format("%Y-%m-%d").to_string();

        if status == TaskStatus::Done {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, completed = ?2, updated = ?3 WHERE id = ?4",
                    rusqlite::params![status.as_str(), now, now, id],
                )
                .map_err(|e| format!("Failed to update task status: {}", e))?;
        } else {
            self.conn
                .execute(
                    "UPDATE tasks SET status = ?1, updated = ?2 WHERE id = ?3",
                    rusqlite::params![status.as_str(), now, id],
                )
                .map_err(|e| format!("Failed to update task status: {}", e))?;
        }

        Ok(())
    }

    /// Delete a task by ID. Fails if other tasks depend on it.
    pub fn delete_task(&self, id: u32) -> Result<(), String> {
        // Check if any tasks depend on this one (app-level check, depends_on is JSON)
        let mut stmt = self
            .conn
            .prepare("SELECT id, depends_on FROM tasks")
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                let task_id: u32 = row.get(0)?;
                let deps_json: String = row.get(1)?;
                Ok((task_id, deps_json))
            })
            .map_err(|e| format!("Failed to query tasks: {}", e))?;

        for row in rows {
            let (task_id, deps_json) = row.map_err(|e| format!("Failed to read row: {}", e))?;
            let deps: Vec<u32> = serde_json::from_str(&deps_json).unwrap_or_default();
            if deps.contains(&id) {
                return Err(format!(
                    "Cannot delete task {}: task {} depends on it",
                    id, task_id
                ));
            }
        }

        // Delete the task (comments cascade via FK)
        let affected = self
            .conn
            .execute("DELETE FROM tasks WHERE id = ?1", [id])
            .map_err(|e| format!("Failed to delete task: {}", e))?;

        if affected == 0 {
            return Err(format!("Task {} does not exist", id));
        }

        Ok(())
    }

    /// Get a task by ID.
    pub fn get_task_by_id(&self, id: u32) -> Option<Task> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, feature, discipline, title, description, status, priority, tags, \
                 depends_on, blocked_by, created, updated, completed, acceptance_criteria, \
                 context_files, output_artifacts, hints, estimated_turns, provenance \
                 FROM tasks WHERE id = ?1",
            )
            .ok()?;

        let task = stmt
            .query_row([id], |row| Ok(self.row_to_task(row)))
            .ok()?;

        let mut task = task;
        task.comments = self.get_comments_for_task(task.id);
        Some(task)
    }

    /// Get all tasks.
    pub fn get_tasks(&self) -> Vec<Task> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, feature, discipline, title, description, status, priority, tags, \
                 depends_on, blocked_by, created, updated, completed, acceptance_criteria, \
                 context_files, output_artifacts, hints, estimated_turns, provenance \
                 FROM tasks ORDER BY id",
            )
            .unwrap();

        let tasks: Vec<Task> = stmt
            .query_map([], |row| Ok(self.row_to_task(row)))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        // Load comments for each task
        let comment_map = self.get_all_comments_by_task();
        tasks
            .into_iter()
            .map(|mut t| {
                t.comments = comment_map.get(&t.id).cloned().unwrap_or_default();
                t
            })
            .collect()
    }

    /// Get enriched tasks with pre-joined feature/discipline display data.
    pub fn get_enriched_tasks(&self) -> Vec<EnrichedTask> {
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

        // First collect all raw rows
        struct RawRow {
            task: Task,
            feature_display_name: String,
            feature_acronym: String,
            discipline_display_name: String,
            discipline_acronym: String,
            discipline_icon: String,
            discipline_color: String,
        }

        let raw_rows: Vec<RawRow> = stmt
            .query_map([], |row| {
                let task = self.row_to_task(row);
                Ok(RawRow {
                    task,
                    feature_display_name: row.get(19)?,
                    feature_acronym: row.get(20)?,
                    discipline_display_name: row.get(21)?,
                    discipline_acronym: row.get(22)?,
                    discipline_icon: row.get(23)?,
                    discipline_color: row.get(24)?,
                })
            })
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();

        // Build task status map for inferred status computation
        let status_map: std::collections::HashMap<u32, TaskStatus> = raw_rows
            .iter()
            .map(|r| (r.task.id, r.task.status))
            .collect();
        // Load all comments
        let comment_map = self.get_all_comments_by_task();

        raw_rows
            .into_iter()
            .map(|r| {
                let inferred_status =
                    Self::compute_inferred_status(r.task.status, &r.task.depends_on, &status_map);
                let comments = comment_map.get(&r.task.id).cloned().unwrap_or_default();

                EnrichedTask {
                    id: r.task.id,
                    feature: r.task.feature,
                    discipline: r.task.discipline,
                    title: r.task.title,
                    description: r.task.description,
                    status: r.task.status,
                    inferred_status,
                    priority: r.task.priority,
                    tags: r.task.tags,
                    depends_on: r.task.depends_on,
                    blocked_by: r.task.blocked_by,
                    created: r.task.created,
                    updated: r.task.updated,
                    completed: r.task.completed,
                    acceptance_criteria: r.task.acceptance_criteria,
                    context_files: r.task.context_files,
                    output_artifacts: r.task.output_artifacts,
                    hints: r.task.hints,
                    estimated_turns: r.task.estimated_turns,
                    provenance: r.task.provenance,
                    comments,
                    feature_display_name: r.feature_display_name,
                    feature_acronym: r.feature_acronym,
                    discipline_display_name: r.discipline_display_name,
                    discipline_acronym: r.discipline_acronym,
                    discipline_icon: r.discipline_icon,
                    discipline_color: r.discipline_color,
                }
            })
            .collect()
    }

    /// Convert a rusqlite Row to a Task (without comments).
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
        }
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
                        .map(|s| *s == TaskStatus::Done)
                        .unwrap_or(false)
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
            .map_err(|e| format!("Failed to prepare query: {}", e))?;

        let deps_map: std::collections::HashMap<u32, Vec<u32>> = stmt
            .query_map([], |row| {
                let id: u32 = row.get(0)?;
                let deps_json: String = row.get(1)?;
                let deps: Vec<u32> = serde_json::from_str(&deps_json).unwrap_or_default();
                Ok((id, deps))
            })
            .map_err(|e| format!("Failed to query: {}", e))?
            .filter_map(|r| r.ok())
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
