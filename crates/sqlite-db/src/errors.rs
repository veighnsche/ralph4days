pub struct RalphError {
    pub code: u16,
    pub message: String,
}

impl std::fmt::Display for RalphError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[R-{:04}] {}", self.code, self.message)
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

macro_rules! ralph_map_err {
    ($code:expr, $msg:expr) => {
        |e| $crate::errors::RalphError {
            code: $code,
            message: format!(concat!($msg, ": {}"), e),
        }.to_string()
    };
}

pub(crate) use ralph_map_err;

pub mod codes {
    pub const DB_OPEN: u16 = 2000;
    pub const DB_READ: u16 = 2100;
    pub const DB_WRITE: u16 = 2200;
    pub const TASK_VALIDATION: u16 = 3000;
    pub const TASK_OPS: u16 = 3100;
    pub const COMMENT_OPS: u16 = 3200;
    pub const FEATURE_OPS: u16 = 4000;
    pub const DISCIPLINE_OPS: u16 = 4100;
}
