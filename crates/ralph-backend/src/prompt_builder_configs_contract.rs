use ralph_macros::ipc_type;
use serde::Deserialize;
use sqlite_db::PromptBuilderConfigInput;

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PromptBuilderConfigGetArgs {
    pub name: String,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PromptBuilderConfigSaveArgs {
    pub config: PromptBuilderConfigInput,
}

#[ipc_type]
#[derive(Debug, Clone, serde::Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PromptBuilderConfigDeleteArgs {
    pub name: String,
}
