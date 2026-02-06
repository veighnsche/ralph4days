impl super::YamlDatabase {
    /// Create a new discipline
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if discipline name already exists
    /// - Returns error if acronym is invalid or already in use
    pub fn create_discipline(
        &mut self,
        name: String,
        display_name: String,
        acronym: String,
        icon: String,
        color: String,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Discipline name cannot be empty".to_string());
        }
        if display_name.trim().is_empty() {
            return Err("Discipline display name cannot be empty".to_string());
        }

        let _lock = self.acquire_lock()?;
        self.load_all()?;

        if self.disciplines.get_all().iter().any(|d| d.name == name) {
            return Err(format!("Discipline '{}' already exists", name));
        }

        crate::acronym::validate_acronym_format(&acronym)?;
        if self
            .disciplines
            .get_all()
            .iter()
            .any(|d| d.acronym == acronym)
        {
            return Err(format!(
                "Acronym '{}' is already used by another discipline",
                acronym
            ));
        }

        self.disciplines.add(crate::Discipline {
            name,
            display_name,
            acronym,
            icon,
            color,
            system_prompt: None,
            skills: vec![],
            conventions: None,
            mcp_servers: vec![],
        });

        self.save_all()?;
        Ok(())
    }

    /// Update an existing discipline
    /// Thread-safe: Uses exclusive file lock
    ///
    /// # Errors
    /// - Returns error if discipline doesn't exist
    /// - Returns error if new acronym is invalid or conflicts with another discipline
    pub fn update_discipline(
        &mut self,
        name: String,
        display_name: String,
        acronym: String,
        icon: String,
        color: String,
    ) -> Result<(), String> {
        if display_name.trim().is_empty() {
            return Err("Discipline display name cannot be empty".to_string());
        }

        let _lock = self.acquire_lock()?;
        self.load_all()?;

        let discipline_index = self
            .disciplines
            .items_mut()
            .iter()
            .position(|d| d.name == name)
            .ok_or_else(|| format!("Discipline '{}' does not exist", name))?;

        crate::acronym::validate_acronym_format(&acronym)?;
        if self
            .disciplines
            .get_all()
            .iter()
            .any(|d| d.acronym == acronym && d.name != name)
        {
            return Err(format!(
                "Acronym '{}' is already used by another discipline",
                acronym
            ));
        }

        let old_disc = &self.disciplines.items_mut()[discipline_index];
        let preserved_name = old_disc.name.clone();
        let preserved_system_prompt = old_disc.system_prompt.clone();
        let preserved_skills = old_disc.skills.clone();
        let preserved_conventions = old_disc.conventions.clone();
        let preserved_mcp_servers = old_disc.mcp_servers.clone();
        self.disciplines.items_mut()[discipline_index] = crate::Discipline {
            name: preserved_name,
            display_name,
            acronym,
            icon,
            color,
            system_prompt: preserved_system_prompt,
            skills: preserved_skills,
            conventions: preserved_conventions,
            mcp_servers: preserved_mcp_servers,
        };

        self.save_all()?;
        Ok(())
    }
}
