pub struct RalphError {
    pub code: u16,
    pub message: String,
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

macro_rules! ralph_err {
    ($code:expr, $($arg:tt)*) => {
        Err($crate::errors::RalphError {
            code: $code,
            message: format!($($arg)*),
        }.to_string())
    };
}

pub(crate) use ralph_err;

#[allow(dead_code)]
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
    pub const PROMPT_MCP: u16 = 6000;
    pub const TERMINAL: u16 = 7000;
    pub const FILESYSTEM: u16 = 8000;
    pub const INTERNAL: u16 = 8100;
}
