use ralph_macros::ipc_type;

#[ipc_type]
pub struct RalphError {
    pub code: u16,
    pub message: String,
}

pub type RalphResult<T> = Result<T, RalphError>;

impl RalphError {
    pub fn new(code: u16, message: String) -> Self {
        let err = Self { code, message };
        tracing::error!(
            error_code = code,
            error_message = %err.message,
            "Ralph error created"
        );
        err
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
        format!(
            "## Error Report

**Error Code:** R-{:04} ({})
**Message:** {}

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
            std::env::consts::OS,
            env!("CARGO_PKG_VERSION"),
            self.message
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
    fn ralph_err(self, code: u16, msg: &str) -> RalphResult<T> {
        self.map_err(|e| RalphError::new(code, format!("{msg}: {e}")))
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
        |e| $crate::RalphError::new($code, format!(concat!($msg, ": {}"), e))
    };
}

pub fn err_string(code: u16, message: impl Into<String>) -> RalphError {
    RalphError::new(code, message.into())
}

pub fn parse_ralph_error(error_str: &str) -> Option<RalphError> {
    let s = error_str.strip_prefix("[R-")?;
    let (code_str, rest) = s.split_once(']')?;
    let code: u16 = code_str.parse().ok()?;
    let message = rest.strip_prefix(' ').unwrap_or(rest).to_owned();
    Some(RalphError { code, message })
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
        let err = RalphError {
            code: codes::DB_OPEN,
            message: "Failed to open database".to_owned(),
        };
        assert_eq!(err.to_string(), "[R-2000] Failed to open database");
    }

    #[test]
    fn test_code_category() {
        assert_eq!(
            RalphError {
                code: codes::PROJECT_PATH,
                message: "test".to_owned()
            }
            .code_category(),
            "PROJECT"
        );
        assert_eq!(
            RalphError {
                code: codes::DB_OPEN,
                message: "test".to_owned()
            }
            .code_category(),
            "DATABASE"
        );
        assert_eq!(
            RalphError {
                code: codes::TERMINAL,
                message: "test".to_owned()
            }
            .code_category(),
            "TERMINAL"
        );
    }

    #[test]
    fn test_parse_ralph_error() {
        let err_str = "[R-2000] Failed to open database";
        let err = parse_ralph_error(err_str).unwrap();
        assert_eq!(err.code, 2000);
        assert_eq!(err.message, "Failed to open database");

        let invalid = "Not a ralph error";
        assert!(parse_ralph_error(invalid).is_none());
    }

    #[test]
    fn test_parse_ralph_error_empty_message() {
        let err = parse_ralph_error("[R-1000]").unwrap();
        assert_eq!(err.code, 1000);
        assert_eq!(err.message, "");
    }

    #[test]
    fn test_parse_ralph_error_invalid_code() {
        assert!(parse_ralph_error("[R-abcd] bad").is_none());
    }

    #[test]
    fn test_github_issue_template() {
        let err = RalphError {
            code: codes::DB_OPEN,
            message: "Failed to open database".to_owned(),
        };
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
        let msg = err
            .ralph_err(codes::DB_WRITE, "Failed to write")
            .unwrap_err();
        let rendered = msg.to_string();
        assert!(rendered.contains("[R-2200]"));
        assert!(rendered.contains("Failed to write: disk full"));
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
        let err = RalphError {
            code: codes::DB_OPEN,
            message: "test".to_owned(),
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"code\":2000"));
        assert!(json.contains("\"message\":\"test\""));
    }
}
