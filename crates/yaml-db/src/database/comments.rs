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

    /// Update a comment's body by index
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if task doesn't exist
    /// - Returns error if comment index is out of bounds
    /// - Returns error if new body is empty
    pub fn update_comment(
        &mut self,
        task_id: u32,
        comment_index: usize,
        body: String,
    ) -> Result<(), String> {
        if body.trim().is_empty() {
            return Err("Comment body cannot be empty".to_string());
        }

        let _lock = self.acquire_lock()?;
        self.load_all()?;

        let task = self
            .tasks
            .items_mut()
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task {} does not exist", task_id))?;

        let num_comments = task.comments.len();
        let comment = task
            .comments
            .get_mut(comment_index)
            .ok_or_else(|| format!("Comment index {} out of bounds (task {} has {} comments)", comment_index, task_id, num_comments))?;

        comment.body = body;

        self.save_all()?;
        Ok(())
    }

    /// Delete a comment by index
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if task doesn't exist
    /// - Returns error if comment index is out of bounds
    pub fn delete_comment(
        &mut self,
        task_id: u32,
        comment_index: usize,
    ) -> Result<(), String> {
        let _lock = self.acquire_lock()?;
        self.load_all()?;

        let task = self
            .tasks
            .items_mut()
            .iter_mut()
            .find(|t| t.id == task_id)
            .ok_or_else(|| format!("Task {} does not exist", task_id))?;

        if comment_index >= task.comments.len() {
            return Err(format!(
                "Comment index {} out of bounds (task {} has {} comments)",
                comment_index, task_id, task.comments.len()
            ));
        }

        task.comments.remove(comment_index);

        self.save_all()?;
        Ok(())
    }
}
