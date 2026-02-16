use crate::prompt_builder_configs_contract::{
    PromptBuilderConfigDeleteArgs, PromptBuilderConfigGetArgs, PromptBuilderConfigSaveArgs,
};
use ralph_contracts::prompt_builder::PromptBuilderConfigData;
use ralph_errors::RalphResult;
use sqlite_db::SqliteDb;

pub fn prompt_builder_config_list(db: &SqliteDb) -> RalphResult<Vec<String>> {
    SqliteDb::list_prompt_builder_configs(db)
}

pub fn prompt_builder_config_get(
    db: &SqliteDb,
    args: PromptBuilderConfigGetArgs,
) -> RalphResult<Option<PromptBuilderConfigData>> {
    db.get_prompt_builder_config(&args.name)
}

pub fn prompt_builder_config_save(
    db: &SqliteDb,
    args: PromptBuilderConfigSaveArgs,
) -> RalphResult<()> {
    db.save_prompt_builder_config(args.config)
}

pub fn prompt_builder_config_delete(
    db: &SqliteDb,
    args: PromptBuilderConfigDeleteArgs,
) -> RalphResult<()> {
    db.delete_prompt_builder_config(&args.name)
}
