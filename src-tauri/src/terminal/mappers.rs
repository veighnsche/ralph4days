use super::contract::TerminalBridgeModelOption;
use super::providers::ModelEntry;

impl From<ModelEntry> for TerminalBridgeModelOption {
    fn from(entry: ModelEntry) -> Self {
        Self {
            name: entry.name,
            display: entry.display,
            description: entry.description,
            session_model: entry.session_model,
            effort_options: entry.effort_options,
        }
    }
}
