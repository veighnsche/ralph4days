use crate::context::PromptContext;
use crate::recipe::Section;

fn build(_ctx: &PromptContext) -> Option<String> {
    None
}

pub fn feature_files() -> Section {
    Section {
        name: "feature_files",
        build,
    }
}
