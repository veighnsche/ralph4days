use crate::SqliteDb;

/// Escape a string for safe inclusion in double-quoted YAML values.
/// Handles: backslashes, double quotes, newlines, tabs.
fn yaml_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

impl SqliteDb {
    /// Export database contents as YAML-formatted text for prompt builder.
    /// Output is deterministic: same DB state always produces identical text.
    pub fn export_prd_yaml(&self) -> Result<String, String> {
        let mut output = String::new();

        // Section 1: metadata
        let meta = self.get_project_info();
        output.push_str("schema_version: \"1.0\"\n");
        output.push_str("project:\n");
        output.push_str(&format!("  title: \"{}\"\n", yaml_escape(&meta.title)));
        if let Some(desc) = &meta.description {
            output.push_str(&format!("  description: \"{}\"\n", yaml_escape(desc)));
        }
        if let Some(created) = &meta.created {
            output.push_str(&format!("  created: \"{}\"\n", created));
        }
        output.push('\n');

        // Section 2: features (sorted by name via ORDER BY)
        let features = self.get_features();
        if !features.is_empty() {
            output.push_str("features:\n");
            for f in &features {
                output.push_str(&format!("- name: \"{}\"\n", yaml_escape(&f.name)));
                output.push_str(&format!(
                    "  display_name: \"{}\"\n",
                    yaml_escape(&f.display_name)
                ));
                if !f.acronym.is_empty() {
                    output.push_str(&format!("  acronym: \"{}\"\n", yaml_escape(&f.acronym)));
                }
                if let Some(desc) = &f.description {
                    output.push_str(&format!("  description: \"{}\"\n", yaml_escape(desc)));
                }
                if let Some(created) = &f.created {
                    output.push_str(&format!("  created: \"{}\"\n", created));
                }
                if !f.knowledge_paths.is_empty() {
                    output.push_str("  knowledge_paths:\n");
                    for kp in &f.knowledge_paths {
                        output.push_str(&format!("  - \"{}\"\n", yaml_escape(kp)));
                    }
                }
                if !f.context_files.is_empty() {
                    output.push_str("  context_files:\n");
                    for cf in &f.context_files {
                        output.push_str(&format!("  - \"{}\"\n", yaml_escape(cf)));
                    }
                }
            }
            output.push('\n');
        }

        // Section 3: disciplines (sorted by name via ORDER BY)
        let disciplines = self.get_disciplines();
        if !disciplines.is_empty() {
            output.push_str("disciplines:\n");
            for d in &disciplines {
                output.push_str(&format!("- name: \"{}\"\n", yaml_escape(&d.name)));
                output.push_str(&format!(
                    "  display_name: \"{}\"\n",
                    yaml_escape(&d.display_name)
                ));
                if !d.acronym.is_empty() {
                    output.push_str(&format!("  acronym: \"{}\"\n", yaml_escape(&d.acronym)));
                }
                output.push_str(&format!("  icon: \"{}\"\n", yaml_escape(&d.icon)));
                output.push_str(&format!("  color: \"{}\"\n", yaml_escape(&d.color)));
            }
            output.push('\n');
        }

        // Section 4: tasks (sorted by id via ORDER BY)
        let tasks = self.get_tasks();
        if !tasks.is_empty() {
            output.push_str("tasks:\n");
            for t in &tasks {
                output.push_str(&format!("- id: {}\n", t.id));
                output.push_str(&format!("  feature: \"{}\"\n", yaml_escape(&t.feature)));
                output.push_str(&format!(
                    "  discipline: \"{}\"\n",
                    yaml_escape(&t.discipline)
                ));
                output.push_str(&format!("  title: \"{}\"\n", yaml_escape(&t.title)));
                if let Some(desc) = &t.description {
                    output.push_str(&format!("  description: \"{}\"\n", yaml_escape(desc)));
                }
                output.push_str(&format!("  status: \"{}\"\n", t.status.as_str()));
                if let Some(priority) = &t.priority {
                    output.push_str(&format!("  priority: \"{}\"\n", priority.as_str()));
                }
                if !t.tags.is_empty() {
                    output.push_str("  tags:\n");
                    for tag in &t.tags {
                        output.push_str(&format!("  - \"{}\"\n", yaml_escape(tag)));
                    }
                }
                if !t.depends_on.is_empty() {
                    output.push_str("  depends_on:\n");
                    for dep in &t.depends_on {
                        output.push_str(&format!("  - {}\n", dep));
                    }
                }
                if let Some(blocked_by) = &t.blocked_by {
                    output.push_str(&format!(
                        "  blocked_by: \"{}\"\n",
                        yaml_escape(blocked_by)
                    ));
                }
                if let Some(created) = &t.created {
                    output.push_str(&format!("  created: \"{}\"\n", created));
                }
                if let Some(updated) = &t.updated {
                    output.push_str(&format!("  updated: \"{}\"\n", updated));
                }
                if let Some(completed) = &t.completed {
                    output.push_str(&format!("  completed: \"{}\"\n", completed));
                }
                if !t.acceptance_criteria.is_empty() {
                    output.push_str("  acceptance_criteria:\n");
                    for ac in &t.acceptance_criteria {
                        output.push_str(&format!("  - \"{}\"\n", yaml_escape(ac)));
                    }
                }
                if !t.context_files.is_empty() {
                    output.push_str("  context_files:\n");
                    for cf in &t.context_files {
                        output.push_str(&format!("  - \"{}\"\n", yaml_escape(cf)));
                    }
                }
                if !t.output_artifacts.is_empty() {
                    output.push_str("  output_artifacts:\n");
                    for oa in &t.output_artifacts {
                        output.push_str(&format!("  - \"{}\"\n", yaml_escape(oa)));
                    }
                }
                if let Some(hints) = &t.hints {
                    output.push_str(&format!("  hints: \"{}\"\n", yaml_escape(hints)));
                }
                if let Some(et) = t.estimated_turns {
                    output.push_str(&format!("  estimated_turns: {}\n", et));
                }
                if let Some(prov) = &t.provenance {
                    output.push_str(&format!("  provenance: \"{}\"\n", prov.as_str()));
                }
                if !t.comments.is_empty() {
                    output.push_str("  comments:\n");
                    for c in &t.comments {
                        output.push_str(&format!(
                            "  - author: \"{}\"\n",
                            c.author.as_str()
                        ));
                        if let Some(atid) = c.agent_task_id {
                            output.push_str(&format!("    agent_task_id: {}\n", atid));
                        }
                        output.push_str(&format!(
                            "    body: \"{}\"\n",
                            yaml_escape(&c.body)
                        ));
                        if let Some(created) = &c.created {
                            output.push_str(&format!("    created: \"{}\"\n", created));
                        }
                    }
                }
            }
        }

        Ok(output)
    }
}
