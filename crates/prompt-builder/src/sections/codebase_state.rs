use crate::context::PromptContext;
use crate::recipe::Section;

fn build(ctx: &PromptContext) -> Option<String> {
    let snap = ctx.codebase_snapshot.as_ref()?;

    if snap.is_empty_project {
        return Some(
            "## Codebase State\n\n\
             This is a greenfield project. No source code files detected yet."
                .to_string(),
        );
    }

    let mut out = format!(
        "## Codebase State\n\n\
         Existing codebase with **{} source files**.",
        snap.total_files
    );

    // Language breakdown
    if !snap.languages.is_empty() {
        out.push_str("\n\nLanguages:");
        let mut langs: Vec<_> = snap.languages.iter().collect();
        langs.sort_by(|a, b| b.1.cmp(a.1));
        for (lang, count) in &langs {
            out.push_str(&format!("\n- {lang}: {count} files"));
        }
    }

    // Directory tree sketch
    if !snap.dir_tree.is_empty() {
        out.push_str("\n\nDirectory structure:");
        for entry in &snap.dir_tree {
            out.push_str(&format!("\n- {entry}/"));
        }
    }

    Some(out)
}

pub fn codebase_state() -> Section {
    Section {
        name: "codebase_state",
        build,
    }
}
