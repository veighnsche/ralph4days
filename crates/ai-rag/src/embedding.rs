//! Embedding utilities for feature comment text.

use sha2::{Digest, Sha256};

/// Build the text that gets embedded for a feature comment.
///
/// Format: `{category}: {body}` or `{category}: {body} (why: {reason})`
pub fn build_embedding_text(category: &str, body: &str, reason: Option<&str>) -> String {
    match reason {
        Some(r) if !r.is_empty() => format!("{category}: {body} (why: {r})"),
        _ => format!("{category}: {body}"),
    }
}

/// SHA256 hash of embedding text, used for dedup (skip re-embedding if unchanged).
pub fn hash_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedding_text_without_reason() {
        let text = build_embedding_text("architecture", "Uses layered architecture", None);
        assert_eq!(text, "architecture: Uses layered architecture");
    }

    #[test]
    fn embedding_text_with_reason() {
        let text = build_embedding_text(
            "convention",
            "Always use snake_case",
            Some("Clippy enforces this"),
        );
        assert_eq!(
            text,
            "convention: Always use snake_case (why: Clippy enforces this)"
        );
    }

    #[test]
    fn embedding_text_with_empty_reason() {
        let text = build_embedding_text("gotcha", "Watch out for lifetimes", Some(""));
        assert_eq!(text, "gotcha: Watch out for lifetimes");
    }

    #[test]
    fn hash_is_deterministic() {
        let h1 = hash_text("hello world");
        let h2 = hash_text("hello world");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_differs_for_different_text() {
        let h1 = hash_text("hello");
        let h2 = hash_text("world");
        assert_ne!(h1, h2);
    }
}
