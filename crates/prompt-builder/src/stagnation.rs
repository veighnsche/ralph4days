use sha2::{Digest, Sha256};

const COMPLETION_MARKER: &str = "<promise>COMPLETE</promise>";

/// SHA256 hash of content for stagnation detection.
pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Check if Claude output contains the completion marker.
pub fn check_completion(output: &str) -> bool {
    output.contains(COMPLETION_MARKER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let a = hash_content("hello world");
        let b = hash_content("hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn hash_differs_for_different_content() {
        let a = hash_content("hello");
        let b = hash_content("world");
        assert_ne!(a, b);
    }

    #[test]
    fn completion_marker_detected() {
        assert!(check_completion("some text <promise>COMPLETE</promise> more text"));
        assert!(!check_completion("some text without marker"));
    }
}
