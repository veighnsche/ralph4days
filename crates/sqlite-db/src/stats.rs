use crate::types::*;
use crate::SqliteDb;

impl SqliteDb {
    /// Get task counts grouped by feature.
    pub fn get_feature_stats(&self) -> Vec<GroupStats> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT f.name, f.display_name, \
                 COUNT(t.id) as total, \
                 SUM(CASE WHEN t.status = 'done' THEN 1 ELSE 0 END) as done, \
                 SUM(CASE WHEN t.status = 'pending' THEN 1 ELSE 0 END) as pending, \
                 SUM(CASE WHEN t.status = 'in_progress' THEN 1 ELSE 0 END) as in_progress, \
                 SUM(CASE WHEN t.status = 'blocked' THEN 1 ELSE 0 END) as blocked, \
                 SUM(CASE WHEN t.status = 'skipped' THEN 1 ELSE 0 END) as skipped \
                 FROM features f \
                 LEFT JOIN tasks t ON f.name = t.feature \
                 GROUP BY f.name, f.display_name \
                 ORDER BY f.name",
            )
            .unwrap();

        stmt.query_map([], |row| {
            Ok(GroupStats {
                name: row.get(0)?,
                display_name: row.get(1)?,
                total: row.get(2)?,
                done: row.get(3)?,
                pending: row.get(4)?,
                in_progress: row.get(5)?,
                blocked: row.get(6)?,
                skipped: row.get(7)?,
            })
        })
        .unwrap()
        .filter_map(std::result::Result::ok)
        .collect()
    }

    /// Get task counts grouped by discipline.
    pub fn get_discipline_stats(&self) -> Vec<GroupStats> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT d.name, d.display_name, \
                 COUNT(t.id) as total, \
                 SUM(CASE WHEN t.status = 'done' THEN 1 ELSE 0 END) as done, \
                 SUM(CASE WHEN t.status = 'pending' THEN 1 ELSE 0 END) as pending, \
                 SUM(CASE WHEN t.status = 'in_progress' THEN 1 ELSE 0 END) as in_progress, \
                 SUM(CASE WHEN t.status = 'blocked' THEN 1 ELSE 0 END) as blocked, \
                 SUM(CASE WHEN t.status = 'skipped' THEN 1 ELSE 0 END) as skipped \
                 FROM disciplines d \
                 LEFT JOIN tasks t ON d.name = t.discipline \
                 GROUP BY d.name, d.display_name \
                 ORDER BY d.name",
            )
            .unwrap();

        stmt.query_map([], |row| {
            Ok(GroupStats {
                name: row.get(0)?,
                display_name: row.get(1)?,
                total: row.get(2)?,
                done: row.get(3)?,
                pending: row.get(4)?,
                in_progress: row.get(5)?,
                blocked: row.get(6)?,
                skipped: row.get(7)?,
            })
        })
        .unwrap()
        .filter_map(std::result::Result::ok)
        .collect()
    }

    /// Get overall project progress.
    pub fn get_project_progress(&self) -> ProjectProgress {
        let (total, done): (u32, u32) = self
            .conn
            .query_row(
                "SELECT COUNT(*), SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) FROM tasks",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap_or((0, 0));

        let percent = if total > 0 { (done * 100) / total } else { 0 };

        ProjectProgress {
            total_tasks: total,
            done_tasks: done,
            progress_percent: percent,
        }
    }

    /// Get sorted unique tags from all tasks.
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut stmt = self.conn.prepare("SELECT tags FROM tasks").unwrap();

        let mut tags = std::collections::BTreeSet::new();
        let rows = stmt
            .query_map([], |row| {
                let json: String = row.get(0)?;
                Ok(json)
            })
            .unwrap();

        for row in rows.flatten() {
            let task_tags: Vec<String> = serde_json::from_str(&row).unwrap_or_default();
            for tag in task_tags {
                tags.insert(tag);
            }
        }

        tags.into_iter().collect()
    }
}
