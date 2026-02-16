use core_macros::ipc_type;

#[ipc_type]
pub struct RalphErrorLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

#[ipc_type]
pub struct RalphErrorContextItem {
    pub key: String,
    #[ts(type = "unknown")]
    pub value: serde_json::Value,
}

#[ipc_type]
pub struct RalphError {
    pub code: u16,
    pub message: String,
    pub location: RalphErrorLocation,
    pub context: Vec<RalphErrorContextItem>,
    pub hint: Option<String>,
}

pub type RalphResult<T> = Result<T, RalphError>;

impl RalphError {
    #[track_caller]
    pub fn new(code: u16, message: String) -> Self {
        let loc = std::panic::Location::caller();
        let err = Self {
            code,
            message,
            location: RalphErrorLocation {
                file: loc.file().to_owned(),
                line: loc.line(),
                column: loc.column(),
            },
            context: Vec::new(),
            hint: None,
        };
        tracing::error!(
            error_code = code,
            error_message = %err.message,
            error_file = %err.location.file,
            error_line = err.location.line,
            error_column = err.location.column,
            "Ralph error created"
        );
        err
    }

    pub fn with_context(
        mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.context.push(RalphErrorContextItem {
            key: key.into(),
            value: value.into(),
        });
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn code_category(&self) -> &str {
        match self.code {
            1000..=1299 => "PROJECT",
            2000..=2299 => "DATABASE",
            3000..=3399 => "TASK",
            4000..=4199 => "FEATURE",
            5000..=5099 => "LOOP_ENGINE",
            7000..=7099 => "TERMINAL",
            8000..=8099 => "FILESYSTEM",
            8100..=8199 => "INTERNAL",
            _ => "UNKNOWN",
        }
    }

    pub fn github_issue_template(&self) -> String {
        let mut context = String::new();
        for item in &self.context {
            let rendered = match &item.value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            context.push_str(&format!("- {}: {}\n", item.key, rendered));
        }

        let hint = self.hint.as_deref().unwrap_or("(none)");

        format!(
            "## Error Report

**Error Code:** R-{:04} ({})
**Message:** {}
**Location:** {}:{}:{}

**Context:**
{}

**Hint:** {}

**Environment:**
- OS: {}
- Ralph Version: {}

**How to Reproduce:**
1.
2.
3.

**Expected Behavior:**


**Actual Behavior:**
{}

**Additional Context:**
<!-- Add any other context about the problem here -->
",
            self.code,
            self.code_category(),
            self.message,
            self.location.file,
            self.location.line,
            self.location.column,
            if context.is_empty() {
                "(none)\n".to_owned()
            } else {
                context
            },
            hint,
            std::env::consts::OS,
            env!("CARGO_PKG_VERSION"),
            self.message
        )
    }

    pub fn github_pr_template(&self) -> String {
        format!(
            "## Pull Request

**Fixes:** R-{:04}

### Summary
- What was broken:
- What changed:

### Error Context (from Ralph)
- Code: R-{:04} ({})
- Message: {}
- Location: {}:{}:{}

### Root Cause

### Changes

### Tests
- [ ] `just verify`
- [ ] `just verify-swap` (if remote/IPC touched)

### Failure Posture
- No new silent fallbacks. Broken invariants still fail loudly.
",
            self.code,
            self.code,
            self.code_category(),
            self.message,
            self.location.file,
            self.location.line,
            self.location.column
        )
    }
}

impl std::fmt::Display for RalphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[R-{:04}] {}", self.code, self.message)
    }
}

pub trait RalphResultExt<T> {
    fn ralph_err(self, code: u16, msg: &str) -> RalphResult<T>;
}

impl<T, E: std::fmt::Display> RalphResultExt<T> for Result<T, E> {
    #[track_caller]
    fn ralph_err(self, code: u16, msg: &str) -> RalphResult<T> {
        self.map_err(|e| {
            let source = e.to_string();
            RalphError::new(code, format!("{msg}: {source}")).with_context("source", source)
        })
    }
}

#[macro_export]
macro_rules! ralph_err {
    ($code:expr, $($arg:tt)*) => {{
        let err = $crate::RalphError::new($code, format!($($arg)*));
        Err(err)
    }};
}

#[macro_export]
macro_rules! ralph_map_err {
    ($code:expr, $msg:expr) => {
        |e| {
            let source = e.to_string();
            $crate::RalphError::new($code, format!(concat!($msg, ": {}"), source))
                .with_context("source", source)
        }
    };
}

#[track_caller]
pub fn err_string(code: u16, message: impl Into<String>) -> RalphError {
    RalphError::new(code, message.into())
}

pub mod codes {
    pub const PROJECT_PATH: u16 = 1000;
    pub const PROJECT_LOCK: u16 = 1100;
    pub const PROJECT_INIT: u16 = 1200;
    pub const DB_OPEN: u16 = 2000;
    pub const DB_READ: u16 = 2100;
    pub const DB_WRITE: u16 = 2200;
    pub const TASK_VALIDATION: u16 = 3000;
    pub const TASK_OPS: u16 = 3100;
    pub const SIGNAL_OPS: u16 = 3300;
    pub const FEATURE_OPS: u16 = 4000;
    pub const DISCIPLINE_OPS: u16 = 4100;
    pub const LOOP_ENGINE: u16 = 5000;
    pub const TERMINAL: u16 = 7000;
    pub const FILESYSTEM: u16 = 8000;
    pub const INTERNAL: u16 = 8100;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = RalphError::new(codes::DB_OPEN, "Failed to open database".to_owned());
        assert_eq!(err.to_string(), "[R-2000] Failed to open database");
    }

    #[test]
    fn test_code_category() {
        assert_eq!(
            RalphError::new(codes::PROJECT_PATH, "test".to_owned()).code_category(),
            "PROJECT"
        );
        assert_eq!(
            RalphError::new(codes::DB_OPEN, "test".to_owned()).code_category(),
            "DATABASE"
        );
        assert_eq!(
            RalphError::new(codes::TERMINAL, "test".to_owned()).code_category(),
            "TERMINAL"
        );
    }

    #[test]
    fn test_github_issue_template() {
        let err = RalphError::new(codes::DB_OPEN, "Failed to open database".to_owned());
        let template = err.github_issue_template();
        assert!(template.contains("R-2000"));
        assert!(template.contains("DATABASE"));
        assert!(template.contains("Failed to open database"));
        assert!(template.contains("## Error Report"));
    }

    #[test]
    fn test_ralph_result_ext_ok() {
        let ok: Result<i32, String> = Ok(42);
        assert_eq!(
            ok.ralph_err(codes::INTERNAL, "should not fire").unwrap(),
            42
        );
    }

    #[test]
    fn test_ralph_result_ext_err() {
        let err: Result<i32, String> = Err("disk full".to_owned());
        let expected_line = line!() + 1;
        let msg = err
            .ralph_err(codes::DB_WRITE, "Failed to write")
            .unwrap_err();
        let rendered = msg.to_string();
        assert!(rendered.contains("[R-2200]"));
        assert!(rendered.contains("Failed to write: disk full"));
        assert_eq!(msg.location.file, file!());
        assert_eq!(msg.location.line, expected_line);
        assert!(msg
            .context
            .iter()
            .any(|i| i.key == "source"
                && i.value == serde_json::Value::String("disk full".to_owned())));
    }

    #[test]
    fn test_ralph_err_macro() {
        let result: Result<(), RalphError> = ralph_err!(codes::DB_OPEN, "test error {}", 42);
        let err = result.unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("[R-2000]"));
        assert!(rendered.contains("test error 42"));
    }

    #[test]
    fn test_err_string() {
        let err = err_string(codes::TERMINAL, "session not found");
        let rendered = err.to_string();
        assert!(rendered.contains("[R-7000]"));
        assert!(rendered.contains("session not found"));
    }

    #[test]
    fn test_ralph_map_err_macro() {
        let result: Result<(), RalphError> =
            Err("original".to_owned()).map_err(ralph_map_err!(codes::DB_WRITE, "wrapping"));
        let err = result.unwrap_err();
        let rendered = err.to_string();
        assert!(rendered.contains("[R-2200]"));
        assert!(rendered.contains("wrapping: original"));
    }

    #[test]
    fn test_serialize() {
        let err = RalphError::new(codes::DB_OPEN, "test".to_owned());
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], codes::DB_OPEN);
        assert_eq!(json["message"], "test");
        assert!(json.get("location").is_some());
        assert!(json["location"]["file"]
            .as_str()
            .unwrap()
            .ends_with("lib.rs"));
        assert!(json["location"]["line"].as_u64().unwrap() > 0);
        assert!(json["location"]["column"].as_u64().unwrap() > 0);
        assert!(json["context"].as_array().unwrap().is_empty());
        assert!(json["hint"].is_null());
    }
}
