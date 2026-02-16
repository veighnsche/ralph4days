use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Dirs to skip when walking the project tree.
const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "build",
    "dist",
    ".next",
    ".venv",
    "venv",
    "__pycache__",
    ".cache",
    "tmp",
    "temp",
    ".tmp",
    "vendor",
    ".idea",
    ".vscode",
    ".ralph",
];

/// Extension → language name mapping for source code files.
fn ext_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        "rs" => Some("Rust"),
        "ts" | "tsx" => Some("TypeScript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("JavaScript"),
        "py" => Some("Python"),
        "go" => Some("Go"),
        "java" => Some("Java"),
        "kt" | "kts" => Some("Kotlin"),
        "swift" => Some("Swift"),
        "c" | "h" => Some("C"),
        "cpp" | "cc" | "cxx" | "hpp" => Some("C++"),
        "cs" => Some("C#"),
        "rb" => Some("Ruby"),
        "php" => Some("PHP"),
        "lua" => Some("Lua"),
        "zig" => Some("Zig"),
        "ex" | "exs" => Some("Elixir"),
        "hs" => Some("Haskell"),
        "ml" | "mli" => Some("OCaml"),
        "html" | "htm" => Some("HTML"),
        "css" | "scss" | "sass" | "less" => Some("CSS"),
        "svelte" => Some("Svelte"),
        "vue" => Some("Vue"),
        "sql" => Some("SQL"),
        "sh" | "bash" | "zsh" | "fish" => Some("Shell"),
        _ => None,
    }
}

/// Lightweight filesystem snapshot of a project's codebase.
#[derive(Debug, Clone)]
pub struct CodebaseSnapshot {
    pub total_files: usize,
    pub languages: BTreeMap<String, usize>,
    pub top_dirs: Vec<String>,
    pub dir_tree: Vec<String>,
    pub is_empty_project: bool,
}

impl Default for CodebaseSnapshot {
    fn default() -> Self {
        Self {
            total_files: 0,
            languages: BTreeMap::new(),
            top_dirs: Vec::new(),
            dir_tree: Vec::new(),
            is_empty_project: true,
        }
    }
}

/// Walk the project directory and produce a snapshot.
/// I/O errors are swallowed — returns Default on failure.
pub fn analyze(project_path: &Path) -> CodebaseSnapshot {
    analyze_inner(project_path).unwrap_or_default()
}

fn analyze_inner(project_path: &Path) -> Option<CodebaseSnapshot> {
    let mut languages: BTreeMap<String, usize> = BTreeMap::new();
    let mut total_files: usize = 0;
    let mut top_dirs: Vec<String> = Vec::new();
    let mut dir_tree_set: BTreeSet<String> = BTreeSet::new();

    // Collect top-level dirs
    let entries = std::fs::read_dir(project_path).ok()?;
    for entry in entries.flatten() {
        let ft = entry.file_type().ok()?;
        if ft.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with('.') && !EXCLUDED_DIRS.contains(&name) {
                    top_dirs.push(name.to_owned());
                }
            }
        }
    }
    top_dirs.sort();

    // Walk the tree
    let mut stack: Vec<(std::path::PathBuf, usize)> = vec![(project_path.to_path_buf(), 0)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = match entry.file_name().to_str() {
                Some(n) => n.to_owned(),
                None => continue,
            };

            if path.is_dir() {
                if name.starts_with('.') || EXCLUDED_DIRS.contains(&name.as_str()) {
                    continue;
                }
                // Record depth-2 dir tree entries (top/sub)
                if depth == 0 && dir_tree_set.len() < 30 {
                    // This is a top-level dir; its children will be depth=1
                    dir_tree_set.insert(name.clone());
                } else if depth == 1 && dir_tree_set.len() < 30 {
                    // depth=1 child: record as "parent/child"
                    let parent = dir
                        .strip_prefix(project_path)
                        .unwrap_or(dir.as_path())
                        .to_string_lossy();
                    dir_tree_set.insert(format!("{parent}/{name}"));
                }
                stack.push((path, depth + 1));
            } else if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if let Some(lang) = ext_to_language(ext) {
                        *languages.entry(lang.to_owned()).or_insert(0) += 1;
                        total_files += 1;
                    }
                }
            }
        }
    }

    let is_empty_project = total_files == 0;
    let dir_tree: Vec<String> = dir_tree_set.into_iter().collect();

    Some(CodebaseSnapshot {
        total_files,
        languages,
        top_dirs,
        dir_tree,
        is_empty_project,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_dir_is_greenfield() {
        let tmp = std::env::temp_dir().join("ralph-snap-test-empty");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();

        let snap = analyze(&tmp);
        assert!(snap.is_empty_project);
        assert_eq!(snap.total_files, 0);

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn nonexistent_dir_returns_default() {
        let snap = analyze(Path::new("/tmp/ralph-snap-test-nonexistent-xyz"));
        assert!(snap.is_empty_project);
        assert_eq!(snap.total_files, 0);
    }

    #[test]
    fn counts_source_files() {
        let tmp = std::env::temp_dir().join("ralph-snap-test-files");
        let _ = fs::remove_dir_all(&tmp);
        let src = tmp.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();
        fs::write(src.join("lib.rs"), "").unwrap();
        fs::write(tmp.join("app.ts"), "").unwrap();
        fs::write(tmp.join("README.md"), "# Hi").unwrap(); // not counted

        let snap = analyze(&tmp);
        assert!(!snap.is_empty_project);
        assert_eq!(snap.total_files, 3);
        assert_eq!(snap.languages.get("Rust"), Some(&2));
        assert_eq!(snap.languages.get("TypeScript"), Some(&1));

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn excludes_node_modules() {
        let tmp = std::env::temp_dir().join("ralph-snap-test-exclude");
        let _ = fs::remove_dir_all(&tmp);
        let nm = tmp.join("node_modules").join("pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("index.js"), "").unwrap();
        fs::create_dir_all(tmp.join("src")).unwrap();
        fs::write(tmp.join("src").join("app.js"), "").unwrap();

        let snap = analyze(&tmp);
        assert_eq!(snap.total_files, 1);
        assert_eq!(snap.languages.get("JavaScript"), Some(&1));

        let _ = fs::remove_dir_all(&tmp);
    }
}
