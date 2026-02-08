pub struct RalphError {
    pub code: u16,
    pub message: String,
}

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
            3000..=3299 => "TASK",
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

impl From<RalphError> for String {
    fn from(e: RalphError) -> Self {
        e.to_string()
    }
}

pub trait ToStringErr<T> {
    fn err_str(self, code: u16) -> Result<T, String>;
}

impl<T, E: std::fmt::Display> ToStringErr<T> for Result<T, E> {
    fn err_str(self, code: u16) -> Result<T, String> {
        self.map_err(|e| RalphError::new(code, e.to_string()).to_string())
    }
}

#[macro_export]
macro_rules! ralph_err {
    ($code:expr, $($arg:tt)*) => {{
        let err = $crate::RalphError::new($code, format!($($arg)*));
        Err(err.to_string())
    }};
}

#[macro_export]
macro_rules! ralph_map_err {
    ($code:expr, $msg:expr) => {
        |e| $crate::RalphError::new($code, format!(concat!($msg, ": {}"), e)).to_string()
    };
}

pub fn err_string(code: u16, message: impl Into<String>) -> String {
    RalphError::new(code, message.into()).to_string()
}

pub fn parse_ralph_error(error_str: &str) -> Option<RalphError> {
    let re = regex::Regex::new(r"^\[R-(\d{4})\] (.*)$").ok()?;
    let caps = re.captures(error_str)?;
    let code: u16 = caps.get(1)?.as_str().parse().ok()?;
    let message = caps.get(2)?.as_str().to_owned();
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
    pub const COMMENT_OPS: u16 = 3200;
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
    fn test_to_string_err_trait() {
        let ok_result: Result<i32, std::io::Error> = Ok(42);
        assert_eq!(ok_result.err_str(codes::INTERNAL).unwrap(), 42);

        let err_result: Result<i32, String> = Err("something broke".to_owned());
        let err = err_result.err_str(codes::INTERNAL).unwrap_err();
        assert!(err.contains("[R-8100]"));
        assert!(err.contains("something broke"));
    }

    #[test]
    fn test_ralph_err_macro() {
        let result: Result<(), String> = ralph_err!(codes::DB_OPEN, "test error {}", 42);
        let err = result.unwrap_err();
        assert!(err.contains("[R-2000]"));
        assert!(err.contains("test error 42"));
    }

    #[test]
    fn test_err_string() {
        let s = err_string(codes::TERMINAL, "session not found");
        assert!(s.contains("[R-7000]"));
        assert!(s.contains("session not found"));
    }

    #[test]
    fn test_ralph_map_err_macro() {
        let result: Result<(), String> =
            Err("original".to_owned()).map_err(ralph_map_err!(codes::DB_WRITE, "wrapping"));
        let err = result.unwrap_err();
        assert!(err.contains("[R-2200]"));
        assert!(err.contains("wrapping: original"));
    }
}
