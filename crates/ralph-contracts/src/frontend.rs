use crate::events::BACKEND_DIAGNOSTIC_EVENT;
use crate::terminal::{TERMINAL_CLOSED_EVENT, TERMINAL_OUTPUT_EVENT};

/// Canonical list of IPC event names the frontend listens to.
///
/// Policy:
/// - Rust constant values are the canonical owner of the string values.
/// - Any add/remove/reorder must be intentional and update the contract tests.
pub const FRONTEND_EVENT_NAMES: &[&str] = &[
    BACKEND_DIAGNOSTIC_EVENT,
    TERMINAL_OUTPUT_EVENT,
    TERMINAL_CLOSED_EVENT,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn frontend_event_names_are_stable() {
        assert_eq!(
            FRONTEND_EVENT_NAMES,
            &[
                BACKEND_DIAGNOSTIC_EVENT,
                TERMINAL_OUTPUT_EVENT,
                TERMINAL_CLOSED_EVENT,
            ]
        );
    }

    #[test]
    fn frontend_event_names_have_no_duplicates() {
        let unique: HashSet<&str> = FRONTEND_EVENT_NAMES.iter().copied().collect();
        assert_eq!(unique.len(), FRONTEND_EVENT_NAMES.len());
    }
}
