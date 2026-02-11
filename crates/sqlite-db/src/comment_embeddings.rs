use crate::SqliteDb;
use ralph_errors::{codes, RalphResultExt};

impl SqliteDb {
    pub fn upsert_comment_embedding(
        &self,
        comment_id: u32,
        embedding: &[f32],
        model: &str,
        hash: &str,
    ) -> Result<(), String> {
        let blob = embedding_to_blob(embedding);
        self.conn
            .execute(
                "INSERT INTO comment_embeddings (comment_id, embedding, embedding_model, embedding_hash)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(comment_id) DO UPDATE SET
                   embedding = excluded.embedding,
                   embedding_model = excluded.embedding_model,
                   embedding_hash = excluded.embedding_hash",
                rusqlite::params![comment_id, blob, model, hash],
            )
            .ralph_err(codes::DB_WRITE, "Failed to upsert comment embedding")?;
        Ok(())
    }

    pub fn delete_comment_embedding(&self, comment_id: u32) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM comment_embeddings WHERE comment_id = ?1",
                [comment_id],
            )
            .ralph_err(codes::DB_WRITE, "Failed to delete comment embedding")?;
        Ok(())
    }

    pub fn search_feature_comments(
        &self,
        feature_name: &str,
        query_embedding: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Vec<ScoredCommentRow> {
        let Ok(mut stmt) = self.conn.prepare(
            "SELECT ce.comment_id, ce.embedding, fc.category, fc.body, fc.summary, fc.reason
             FROM comment_embeddings ce
             JOIN feature_comments fc ON fc.id = ce.comment_id
             JOIN features f ON fc.feature_id = f.id
             WHERE f.name = ?1",
        ) else {
            return vec![];
        };

        let Ok(rows) = stmt.query_map([feature_name], |row| {
            let comment_id: u32 = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            let category: String = row.get(2)?;
            let body: String = row.get(3)?;
            let summary: Option<String> = row.get(4)?;
            let reason: Option<String> = row.get(5)?;
            Ok((comment_id, blob, category, body, summary, reason))
        }) else {
            return vec![];
        };

        let mut results: Vec<ScoredCommentRow> = rows
            .filter_map(Result::ok)
            .filter_map(|(comment_id, blob, category, body, summary, reason)| {
                let stored = blob_to_embedding(&blob)?;
                let score = cosine_similarity(query_embedding, &stored);
                if score >= min_score {
                    Some(ScoredCommentRow {
                        comment_id,
                        category,
                        body,
                        summary,
                        reason,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        results
    }

    pub fn has_comment_embedding(&self, comment_id: u32) -> bool {
        self.conn
            .query_row(
                "SELECT 1 FROM comment_embeddings WHERE comment_id = ?1",
                [comment_id],
                |_| Ok(()),
            )
            .is_ok()
    }

    pub fn get_embedding_hash(&self, comment_id: u32) -> Option<String> {
        self.conn
            .query_row(
                "SELECT embedding_hash FROM comment_embeddings WHERE comment_id = ?1",
                [comment_id],
                |row| row.get(0),
            )
            .ok()
    }
}

#[derive(Debug, Clone)]
pub struct ScoredCommentRow {
    pub comment_id: u32,
    pub category: String,
    pub body: String,
    pub summary: Option<String>,
    pub reason: Option<String>,
    pub score: f32,
}

fn embedding_to_blob(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn blob_to_embedding(blob: &[u8]) -> Option<Vec<f32>> {
    if blob.len() % 4 != 0 {
        return None;
    }
    Some(
        blob.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect(),
    )
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0_f32;
    let mut norm_a = 0.0_f32;
    let mut norm_b = 0.0_f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_blob_roundtrip() {
        let original = vec![0.1_f32, 0.2, -0.5, 1.0, 0.0];
        let blob = embedding_to_blob(&original);
        let restored = blob_to_embedding(&blob).unwrap();
        assert_eq!(original, restored);
    }

    #[test]
    fn cosine_identical_vectors() {
        let v = vec![1.0, 2.0, 3.0];
        let score = cosine_similarity(&v, &v);
        assert!((score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn cosine_orthogonal_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let score = cosine_similarity(&a, &b);
        assert!(score.abs() < 1e-6);
    }

    #[test]
    fn cosine_opposite_vectors() {
        let a = vec![1.0, 0.0];
        let b = vec![-1.0, 0.0];
        let score = cosine_similarity(&a, &b);
        assert!((score - (-1.0)).abs() < 1e-6);
    }

    #[test]
    fn upsert_and_search() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        db.create_feature(crate::FeatureInput {
            name: "auth".to_owned(),
            display_name: "Auth".to_owned(),
            acronym: "AUTH".to_owned(),
            description: None,
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Use JWT not sessions".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();

        let features = db.get_features();
        let comment_id = features[0].comments[0].id;

        let embedding = vec![0.5_f32; 768];
        db.upsert_comment_embedding(comment_id, &embedding, "nomic-embed-text", "abc123")
            .unwrap();

        assert!(db.has_comment_embedding(comment_id));
        assert_eq!(db.get_embedding_hash(comment_id), Some("abc123".to_owned()));

        let query = vec![0.5_f32; 768];
        let results = db.search_feature_comments("auth", &query, 10, 0.0);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].body, "Use JWT not sessions");
        assert!((results[0].score - 1.0).abs() < 1e-6);
    }

    #[test]
    fn search_respects_min_score() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        db.create_feature(crate::FeatureInput {
            name: "auth".to_owned(),
            display_name: "Auth".to_owned(),
            acronym: "AUTH".to_owned(),
            description: None,
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Low relevance".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();

        let features = db.get_features();
        let comment_id = features[0].comments[0].id;

        let embedding = vec![1.0, 0.0, 0.0];
        db.upsert_comment_embedding(comment_id, &embedding, "test", "hash1")
            .unwrap();

        let query = vec![0.0, 1.0, 0.0];
        let results = db.search_feature_comments("auth", &query, 10, 0.4);
        assert!(results.is_empty());
    }

    #[test]
    fn cascade_delete_removes_embedding() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        db.create_feature(crate::FeatureInput {
            name: "auth".to_owned(),
            display_name: "Auth".to_owned(),
            acronym: "AUTH".to_owned(),
            description: None,
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Test".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();
        let features = db.get_features();
        let comment_id = features[0].comments[0].id;

        db.upsert_comment_embedding(comment_id, &[0.5; 768], "test", "hash1")
            .unwrap();
        assert!(db.has_comment_embedding(comment_id));

        db.delete_feature_comment("auth", comment_id).unwrap();
        assert!(!db.has_comment_embedding(comment_id));
    }

    #[test]
    fn delete_embedding_direct() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        db.create_feature(crate::FeatureInput {
            name: "auth".to_owned(),
            display_name: "Auth".to_owned(),
            acronym: "AUTH".to_owned(),
            ..Default::default()
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Test delete".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();
        let comment_id = db.get_features()[0].comments[0].id;

        db.upsert_comment_embedding(comment_id, &[0.5; 768], "test", "hash1")
            .unwrap();
        assert!(db.has_comment_embedding(comment_id));

        db.delete_comment_embedding(comment_id).unwrap();
        assert!(!db.has_comment_embedding(comment_id));
    }

    #[test]
    fn upsert_overwrites_embedding() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        db.create_feature(crate::FeatureInput {
            name: "auth".to_owned(),
            display_name: "Auth".to_owned(),
            acronym: "AUTH".to_owned(),
            ..Default::default()
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Test upsert".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();
        let comment_id = db.get_features()[0].comments[0].id;

        db.upsert_comment_embedding(comment_id, &[0.1; 768], "test", "hash_old")
            .unwrap();
        assert_eq!(
            db.get_embedding_hash(comment_id),
            Some("hash_old".to_owned())
        );

        db.upsert_comment_embedding(comment_id, &[0.9; 768], "test", "hash_new")
            .unwrap();
        assert_eq!(
            db.get_embedding_hash(comment_id),
            Some("hash_new".to_owned())
        );
    }

    #[test]
    fn search_multiple_features_isolated() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        for (name, display, acronym) in [("auth", "Auth", "AUTH"), ("billing", "Billing", "BILL")] {
            db.create_feature(crate::FeatureInput {
                name: name.to_owned(),
                display_name: display.to_owned(),
                acronym: acronym.to_owned(),
                ..Default::default()
            })
            .unwrap();
        }

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Auth only".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "billing".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Billing only".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();

        let features = db.get_features();
        let auth_cid = features.iter().find(|f| f.name == "auth").unwrap().comments[0].id;
        let bill_cid = features
            .iter()
            .find(|f| f.name == "billing")
            .unwrap()
            .comments[0]
            .id;

        let emb = vec![0.5_f32; 768];
        db.upsert_comment_embedding(auth_cid, &emb, "test", "h1")
            .unwrap();
        db.upsert_comment_embedding(bill_cid, &emb, "test", "h2")
            .unwrap();

        let auth_results = db.search_feature_comments("auth", &emb, 10, 0.0);
        assert_eq!(auth_results.len(), 1);
        assert_eq!(auth_results[0].body, "Auth only");

        let billing_results = db.search_feature_comments("billing", &emb, 10, 0.0);
        assert_eq!(billing_results.len(), 1);
        assert_eq!(billing_results[0].body, "Billing only");
    }

    #[test]
    fn search_ordering_by_score() {
        let db = SqliteDb::open_in_memory(None).unwrap();
        db.create_feature(crate::FeatureInput {
            name: "auth".to_owned(),
            display_name: "Auth".to_owned(),
            acronym: "AUTH".to_owned(),
            ..Default::default()
        })
        .unwrap();

        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "gotcha".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "Low match".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();
        db.add_feature_comment(crate::AddFeatureCommentInput {
            feature_name: "auth".to_owned(),
            category: "convention".to_owned(),

            discipline: None,
            agent_task_id: None,
            body: "High match".to_owned(),
            summary: None,
            reason: None,
            source_iteration: None,
        })
        .unwrap();

        let features = db.get_features();
        let comments = &features.iter().find(|f| f.name == "auth").unwrap().comments;
        let cid_low = comments.iter().find(|c| c.body == "Low match").unwrap().id;
        let cid_high = comments.iter().find(|c| c.body == "High match").unwrap().id;

        // Low match: orthogonal to query
        db.upsert_comment_embedding(cid_low, &[1.0, 0.0, 0.0], "test", "h1")
            .unwrap();
        // High match: identical to query
        db.upsert_comment_embedding(cid_high, &[0.0, 1.0, 0.0], "test", "h2")
            .unwrap();

        let query = vec![0.0, 1.0, 0.0];
        let results = db.search_feature_comments("auth", &query, 10, 0.0);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].body, "High match");
        assert_eq!(results[1].body, "Low match");
        assert!(results[0].score > results[1].score);
    }
}
