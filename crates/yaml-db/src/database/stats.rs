impl super::YamlDatabase {
    /// Get task counts grouped by feature
    pub fn get_feature_stats(&self) -> Vec<crate::GroupStats> {
        use std::collections::HashMap;
        let mut stats: HashMap<String, crate::GroupStats> = HashMap::new();

        for feature in self.features.get_all() {
            stats.insert(
                feature.name.clone(),
                crate::GroupStats {
                    name: feature.name.clone(),
                    display_name: feature.display_name.clone(),
                    total: 0,
                    done: 0,
                    pending: 0,
                    in_progress: 0,
                    blocked: 0,
                    skipped: 0,
                },
            );
        }

        for task in self.tasks.get_all() {
            if let Some(entry) = stats.get_mut(&task.feature) {
                entry.total += 1;
                match task.status {
                    crate::TaskStatus::Done => entry.done += 1,
                    crate::TaskStatus::Pending => entry.pending += 1,
                    crate::TaskStatus::InProgress => entry.in_progress += 1,
                    crate::TaskStatus::Blocked => entry.blocked += 1,
                    crate::TaskStatus::Skipped => entry.skipped += 1,
                }
            }
        }

        let mut result: Vec<_> = stats.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Get task counts grouped by discipline
    pub fn get_discipline_stats(&self) -> Vec<crate::GroupStats> {
        use std::collections::HashMap;
        let mut stats: HashMap<String, crate::GroupStats> = HashMap::new();

        for discipline in self.disciplines.get_all() {
            stats.insert(
                discipline.name.clone(),
                crate::GroupStats {
                    name: discipline.name.clone(),
                    display_name: discipline.display_name.clone(),
                    total: 0,
                    done: 0,
                    pending: 0,
                    in_progress: 0,
                    blocked: 0,
                    skipped: 0,
                },
            );
        }

        for task in self.tasks.get_all() {
            if let Some(entry) = stats.get_mut(&task.discipline) {
                entry.total += 1;
                match task.status {
                    crate::TaskStatus::Done => entry.done += 1,
                    crate::TaskStatus::Pending => entry.pending += 1,
                    crate::TaskStatus::InProgress => entry.in_progress += 1,
                    crate::TaskStatus::Blocked => entry.blocked += 1,
                    crate::TaskStatus::Skipped => entry.skipped += 1,
                }
            }
        }

        let mut result: Vec<_> = stats.into_values().collect();
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Get overall project progress
    pub fn get_project_progress(&self) -> crate::ProjectProgress {
        let tasks = self.tasks.get_all();
        let total = tasks.len() as u32;
        let done = tasks
            .iter()
            .filter(|t| t.status == crate::TaskStatus::Done)
            .count() as u32;
        let percent = if total > 0 {
            (done * 100) / total
        } else {
            0
        };
        crate::ProjectProgress {
            total_tasks: total,
            done_tasks: done,
            progress_percent: percent,
        }
    }

    /// Get sorted unique tags from all tasks
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags = std::collections::BTreeSet::new();
        for task in self.tasks.get_all() {
            for tag in &task.tags {
                tags.insert(tag.clone());
            }
        }
        tags.into_iter().collect()
    }
}
