use ralph_macros::ipc_type;

#[ipc_type]
pub struct AgentSessionsByIdArgs {
    pub id: String,
}
