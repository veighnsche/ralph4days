impl super::YamlDatabase {
    /// Create a new feature
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if feature name already exists
    /// - Returns error if acronym is invalid or already in use
    pub fn create_feature(
        &mut self,
        name: String,
        display_name: String,
        acronym: String,
        description: Option<String>,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Feature name cannot be empty".to_string());
        }
        if display_name.trim().is_empty() {
            return Err("Feature display name cannot be empty".to_string());
        }

        let _lock = self.acquire_lock()?;
        self.load_all()?;

        if self.features.get_all().iter().any(|f| f.name == name) {
            return Err(format!("Feature '{}' already exists", name));
        }

        crate::acronym::validate_acronym_format(&acronym)?;
        if self.features.get_all().iter().any(|f| f.acronym == acronym) {
            return Err(format!(
                "Acronym '{}' is already used by another feature",
                acronym
            ));
        }

        self.features.add(crate::Feature {
            name,
            display_name,
            acronym,
            description,
            created: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            knowledge_paths: vec![],
            context_files: vec![],
        });

        self.save_all()?;
        Ok(())
    }

    /// Update an existing feature
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if feature doesn't exist
    /// - Returns error if new acronym is invalid or conflicts with another feature
    pub fn update_feature(
        &mut self,
        name: String,
        display_name: String,
        acronym: String,
        description: Option<String>,
    ) -> Result<(), String> {
        if display_name.trim().is_empty() {
            return Err("Feature display name cannot be empty".to_string());
        }

        let _lock = self.acquire_lock()?;
        self.load_all()?;

        let feature_index = self
            .features
            .items_mut()
            .iter()
            .position(|f| f.name == name)
            .ok_or_else(|| format!("Feature '{}' does not exist", name))?;

        crate::acronym::validate_acronym_format(&acronym)?;
        if self
            .features
            .get_all()
            .iter()
            .any(|f| f.acronym == acronym && f.name != name)
        {
            return Err(format!(
                "Acronym '{}' is already used by another feature",
                acronym
            ));
        }

        let old_feature = &self.features.items_mut()[feature_index];
        self.features.items_mut()[feature_index] = crate::Feature {
            name: old_feature.name.clone(),
            display_name,
            acronym,
            description,
            created: old_feature.created.clone(),
            knowledge_paths: old_feature.knowledge_paths.clone(),
            context_files: old_feature.context_files.clone(),
        };

        self.save_all()?;
        Ok(())
    }

    /// Delete a feature by name
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if feature doesn't exist
    /// - Returns error if any tasks reference this feature
    pub fn delete_feature(&mut self, name: String) -> Result<(), String> {
        let _lock = self.acquire_lock()?;
        self.load_all()?;

        // Check if any tasks reference this feature
        for task in self.tasks.get_all() {
            if task.feature == name {
                return Err(format!(
                    "Cannot delete feature '{}': task {} ('{}') belongs to it",
                    name, task.id, task.title
                ));
            }
        }

        let initial_len = self.features.items_mut().len();
        self.features.items_mut().retain(|f| f.name != name);

        if self.features.items_mut().len() == initial_len {
            return Err(format!("Feature '{}' does not exist", name));
        }

        self.save_all()?;
        Ok(())
    }
}
