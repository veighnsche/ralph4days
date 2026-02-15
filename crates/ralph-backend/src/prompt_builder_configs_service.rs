use crate::prompt_builder_configs_contract::{
    PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs, PromptBuilderConfigSaveArgs,
};
use sqlite_db::{PromptBuilderConfigData, SqliteDb};

pub fn prompt_builder_config_list(db: &SqliteDb) -> Result<Vec<String>, String> {
    SqliteDb::list_prompt_builder_configs(db)
}

pub fn prompt_builder_config_get(
    db: &SqliteDb,
    args: PromptBuilderConfigGetArgs,
) -> Result<Option<PromptBuilderConfigData>, String> {
    db.get_prompt_builder_config(&args.name)
}

pub fn prompt_builder_config_save(
    db: &SqliteDb,
    args: PromptBuilderConfigSaveArgs,
) -> Result<(), String> {
    db.save_prompt_builder_config(args.config)
}

pub fn prompt_builder_config_delete(
    db: &SqliteDb,
    args: PromptBuilderConfigDeleteArgs,
) -> Result<(), String> {
    db.delete_prompt_builder_config(&args.name)
}
