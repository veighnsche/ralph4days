use ralph_macros::ipc_type;
use sqlite_db::PromptBuilderConfigInput;

#[ipc_type]
pub struct PromptBuilderConfigGetArgs {
    pub name: String,
}

#[ipc_type]
pub struct PromptBuilderConfigSaveArgs {
    pub config: PromptBuilderConfigInput,
}

#[ipc_type]
pub struct PromptBuilderConfigDeleteArgs {
    pub name: String,
}
