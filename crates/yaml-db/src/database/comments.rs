impl super::YamlDatabase {
    /// Add a comment to an existing task
    /// Thread-safe: Uses exclusive file lock
    pub fn add_comment(
        &mut self,
        task_id: u32,
        author: crate::CommentAuthor,
        agent_task_id: Option<u32>,
        body: String,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return Err("Comment body cannot be empty".to_string());
        }
        if author == crate::CommentAuthor::Agent && agent_task_id.is_none() {
            return Err("agent_task_id is required for agent comments".to_string());
        }
        if author == crate::CommentAuthor::Human && agent_task_id.is_some() {
            return Err("agent_task_id must not be set for human comments".to_string());
        }

        let _lock = self.acquire_lock()?;
        self.load_all()?;

        let task = self
            .tasks
            .items_mut()
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task {} does not exist", task_id))?;

        task.comments.push(crate::TaskComment {
            author,
            agent_task_id,
            body,
            created: Some(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        });

        self.save_all()?;
        Ok(())
    }
}
