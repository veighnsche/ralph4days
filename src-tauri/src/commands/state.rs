#[cfg(not(mobile))]
#[path = "state_desktop.rs"]
mod imp;

#[cfg(mobile)]
#[path = "state_mobile.rs"]
mod imp;

pub use imp::AppState;
#[cfg(not(mobile))]
pub(crate) use imp::{with_db, CommandContext};
