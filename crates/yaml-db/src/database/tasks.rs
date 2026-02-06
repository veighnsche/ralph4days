use crate::{Task, TaskStatus};

impl super::YamlDatabase {
    /// Create a new task with automatic ID assignment and feature/discipline creation
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if feature or discipline name is empty
    /// - Returns error if title is empty
    /// - Returns error if dependency references non-existent task
    pub fn create_task(&mut self, task: super::TaskInput) -> Result<u32, String> {
        // 0. Validate input
        if task.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if task.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if task.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        // 1. Acquire exclusive lock (blocks until available)
        let _lock = self.acquire_lock()?;

        // 2. Reload all files from disk (ensure fresh state)
        self.load_all()?;

        // 3. Validate feature exists
        if !self.features.get_all().iter().any(|f| f.name == task.feature) {
            return Err(format!(
                "Feature '{}' does not exist. Create it first.",
                task.feature
            ));
        }

        // 4. Validate discipline exists
        if !self
            .disciplines
            .get_all()
            .iter()
            .any(|d| d.name == task.discipline)
        {
            return Err(format!(
                "Discipline '{}' does not exist. Create it first.",
                task.discipline
            ));
        }

        // 5. Assign next ID (uses global counter)
        let next_id = self.metadata.get_next_id(self.tasks.get_all());

        // 6. Create task with assigned ID
        let new_task = Task {
            id: next_id,
            feature: task.feature,
            discipline: task.discipline,
            title: task.title,
            description: task.description,
            status: TaskStatus::Pending,
            priority: task.priority,
            tags: task.tags,
            depends_on: task.depends_on,
            blocked_by: None,
            created: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            updated: None,
            completed: None,
            acceptance_criteria: task.acceptance_criteria.unwrap_or_default(),
            context_files: task.context_files,
            output_artifacts: task.output_artifacts,
            hints: task.hints,
            estimated_turns: task.estimated_turns,
            provenance: task.provenance,
            comments: vec![],
        };

        // 7. Validate dependencies exist
        self.validate_dependencies(&new_task)?;

        // 8. Add task to tasks file
        self.tasks.add_task(new_task);

        // 9. Update counters
        self.metadata.rebuild_counters(self.tasks.get_all());

        // 10. Write all files atomically
        self.save_all()?;

        // 11. Lock auto-released on drop
        Ok(next_id)
    }

    /// Update an existing task
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if task ID doesn't exist
    /// - Returns error if validation fails (empty fields, invalid dependencies)
    pub fn update_task(&mut self, id: u32, update: super::TaskInput) -> Result<(), String> {
        // 0. Validate input
        if update.feature.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if update.discipline.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if update.title.trim().is_empty() {
            return Err("Task title cannot be empty".to_string());
        }

        // 1. Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // 2. Reload all files from disk
        self.load_all()?;

        // 3. Find the task to update
        let task_index = self
            .tasks
            .items_mut()
            .iter()
            .position(|t| t.id == id)
            .ok_or_else(|| format!("Task {} does not exist", id))?;

        // 4. Validate feature exists
        if !self.features.get_all().iter().any(|f| f.name == update.feature) {
            return Err(format!(
                "Feature '{}' does not exist. Create it first.",
                update.feature
            ));
        }

        // 5. Validate discipline exists
        if !self
            .disciplines
            .get_all()
            .iter()
            .any(|d| d.name == update.discipline)
        {
            return Err(format!(
                "Discipline '{}' does not exist. Create it first.",
                update.discipline
            ));
        }

        // 6. Create updated task (preserve ID, status, and timestamps)
        let old_task = &self.tasks.items_mut()[task_index];
        let updated_task = Task {
            id,
            feature: update.feature,
            discipline: update.discipline,
            title: update.title,
            description: update.description,
            status: old_task.status, // Preserve status
            priority: update.priority,
            tags: update.tags,
            depends_on: update.depends_on,
            blocked_by: old_task.blocked_by.clone(), // Preserve blocked_by
            created: old_task.created.clone(),       // Preserve created
            updated: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            completed: old_task.completed.clone(), // Preserve completed
            acceptance_criteria: update.acceptance_criteria.unwrap_or_default(),
            context_files: update.context_files,
            output_artifacts: update.output_artifacts,
            hints: update.hints,
            estimated_turns: update.estimated_turns,
            provenance: old_task.provenance,  // Preserve
            comments: old_task.comments.clone(), // Preserve
        };

        // 7. Validate dependencies exist (including checking for cycles)
        self.validate_dependencies_with_cycles(id, &updated_task)?;

        // 8. Update the task in place
        self.tasks.items_mut()[task_index] = updated_task;

        // 9. Update counters
        self.metadata.rebuild_counters(self.tasks.get_all());

        // 10. Write all files atomically
        self.save_all()?;

        Ok(())
    }

    /// Delete a task by ID
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if task ID doesn't exist
    /// - Returns error if other tasks depend on this task
    pub fn delete_task(&mut self, id: u32) -> Result<(), String> {
        // 1. Acquire exclusive lock
        let _lock = self.acquire_lock()?;

        // 2. Reload all files from disk
        self.load_all()?;

        // 3. Check if any tasks depend on this one
        for task in self.tasks.get_all() {
            if task.depends_on.contains(&id) {
                return Err(format!(
                    "Cannot delete task {}: task {} depends on it",
                    id, task.id
                ));
            }
        }

        // 4. Find and remove the task
        let initial_len = self.tasks.items_mut().len();
        self.tasks.items_mut().retain(|t| t.id != id);

        if self.tasks.items_mut().len() == initial_len {
            return Err(format!("Task {} does not exist", id));
        }

        // 5. Update counters
        self.metadata.rebuild_counters(self.tasks.get_all());

        // 6. Write all files atomically
        self.save_all()?;

        Ok(())
    }

    /// Get a task by ID
    pub fn get_task_by_id(&self, id: u32) -> Option<&Task> {
        self.tasks.get_all().iter().find(|t| t.id == id)
    }

    /// Compute the inferred status for a task based on its actual status and dependencies
    fn compute_inferred_status(&self, task: &Task) -> crate::InferredTaskStatus {
        use crate::InferredTaskStatus;

        match task.status {
            TaskStatus::InProgress => InferredTaskStatus::InProgress,
            TaskStatus::Done => InferredTaskStatus::Done,
            TaskStatus::Skipped => InferredTaskStatus::Skipped,
            TaskStatus::Blocked => InferredTaskStatus::ExternallyBlocked,
            TaskStatus::Pending => {
                let all_deps_met = task.depends_on.iter().all(|dep_id| {
                    self.get_task_by_id(*dep_id)
                        .map(|dep| dep.status == TaskStatus::Done)
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

    /// Get tasks joined with feature/discipline display data
    pub fn get_enriched_tasks(&self) -> Vec<crate::EnrichedTask> {
        let features = self.features.get_all();
        let disciplines = self.disciplines.get_all();

        self.tasks
            .get_all()
            .iter()
            .map(|task| {
                let feature = features.iter().find(|f| f.name == task.feature);
                let discipline = disciplines.iter().find(|d| d.name == task.discipline);
                let inferred_status = self.compute_inferred_status(task);

                crate::EnrichedTask {
                    id: task.id,
                    feature: task.feature.clone(),
                    discipline: task.discipline.clone(),
                    title: task.title.clone(),
                    description: task.description.clone(),
                    status: task.status,
                    inferred_status,
                    priority: task.priority,
                    tags: task.tags.clone(),
                    depends_on: task.depends_on.clone(),
                    blocked_by: task.blocked_by.clone(),
                    created: task.created.clone(),
                    updated: task.updated.clone(),
                    completed: task.completed.clone(),
                    acceptance_criteria: task.acceptance_criteria.clone(),
                    context_files: task.context_files.clone(),
                    output_artifacts: task.output_artifacts.clone(),
                    hints: task.hints.clone(),
                    estimated_turns: task.estimated_turns,
                    provenance: task.provenance,
                    comments: task.comments.clone(),
                    feature_display_name: feature
                        .map(|f| f.display_name.clone())
                        .unwrap_or_else(|| task.feature.clone()),
                    feature_acronym: feature
                        .map(|f| f.acronym.clone())
                        .unwrap_or_else(|| task.feature.clone()),
                    discipline_display_name: discipline
                        .map(|d| d.display_name.clone())
                        .unwrap_or_else(|| task.discipline.clone()),
                    discipline_acronym: discipline
                        .map(|d| d.acronym.clone())
                        .unwrap_or_else(|| task.discipline.clone()),
                    discipline_icon: discipline
                        .map(|d| d.icon.clone())
                        .unwrap_or_else(|| "Circle".to_string()),
                    discipline_color: discipline
                        .map(|d| d.color.clone())
                        .unwrap_or_else(|| "#94a3b8".to_string()),
                }
            })
            .collect()
    }

    /// Validate that all task dependencies reference existing tasks
    fn validate_dependencies(&self, task: &Task) -> Result<(), String> {
        for dep_id in &task.depends_on {
            if !self.tasks.has_task(*dep_id) {
                return Err(format!("Dependency task {} does not exist", dep_id));
            }
        }
        Ok(())
    }

    /// Validate dependencies including circular dependency detection
    fn validate_dependencies_with_cycles(
        &self,
        task_id: u32,
        task: &Task,
    ) -> Result<(), String> {
        self.validate_dependencies(task)?;

        if task.depends_on.contains(&task_id) {
            return Err(format!(
                "Task {} cannot depend on itself",
                task_id
            ));
        }

        for dep_id in &task.depends_on {
            if self.has_circular_dependency(task_id, *dep_id)? {
                return Err(format!(
                    "Circular dependency detected: task {} would create a cycle with task {}",
                    task_id, dep_id
                ));
            }
        }

        Ok(())
    }

    /// Recursively check if adding a dependency would create a cycle
    fn has_circular_dependency(&self, task_id: u32, dep_id: u32) -> Result<bool, String> {
        use std::collections::HashSet;

        let mut visited = HashSet::new();
        let mut stack = vec![dep_id];

        while let Some(current_id) = stack.pop() {
            if current_id == task_id {
                return Ok(true);
            }

            if !visited.insert(current_id) {
                continue;
            }

            if let Some(current_task) = self.tasks.get_all().iter().find(|t| t.id == current_id) {
                for &next_dep in &current_task.depends_on {
                    stack.push(next_dep);
                }
            }
        }

        Ok(false)
    }
}
