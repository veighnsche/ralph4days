use ralph_macros::ipc_type;
use serde::{Deserialize, Serialize};

pub const BACKEND_DIAGNOSTIC_EVENT: &str = "backend-diagnostic";

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendDiagnosticLevel {
    Warning,
    Error,
}

#[ipc_type]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackendDiagnosticEvent {
    pub level: BackendDiagnosticLevel,
    pub source: String,
    pub code: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_diagnostic_event_name_is_stable() {
        assert_eq!(BACKEND_DIAGNOSTIC_EVENT, "backend-diagnostic");
    }

    #[test]
    fn backend_diagnostic_event_serializes_expected_shape() {
        let payload = BackendDiagnosticEvent {
            level: BackendDiagnosticLevel::Warning,
            source: "app-state".to_owned(),
            code: "xdg-resolve-fallback".to_owned(),
            message: "Failed to resolve XDG directories".to_owned(),
        };

        let json = serde_json::to_value(payload).unwrap();
        assert_eq!(json["level"], "warning");
        assert_eq!(json["source"], "app-state");
        assert_eq!(json["code"], "xdg-resolve-fallback");
        assert_eq!(json["message"], "Failed to resolve XDG directories");
    }
}
