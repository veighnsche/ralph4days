use ralph_macros::ipc_type;

#[ipc_type]
pub struct ProjectLockSetArgs {
    pub path: String,
}
