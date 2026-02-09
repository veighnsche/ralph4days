use std::collections::HashMap;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let stacks: &[(u8, &str)] = &[
        (1, "01_generic"),
        (2, "02_desktop"),
        (3, "03_saas"),
        (4, "04_mobile"),
    ];

    // (stack_id, discipline_name) -> absolute path to latest PNG
    let mut images: Vec<(u8, String, PathBuf)> = Vec::new();

    for &(stack_id, stack_dir) in stacks {
        let images_dir = manifest_dir
            .join("src/defaults/disciplines")
            .join(stack_dir)
            .join("images");

        if !images_dir.exists() {
            continue;
        }

        // Group PNGs by discipline prefix (e.g. "00_implementation")
        let mut by_discipline: HashMap<String, Vec<(String, PathBuf)>> = HashMap::new();

        let Ok(entries) = std::fs::read_dir(&images_dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("png") {
                continue;
            }
            let filename = path.file_stem().unwrap().to_string_lossy().to_string();

            // Pattern: NN_name_steps_WxH_base36timestamp
            // Extract discipline name: everything between first _ and the _steps_ part
            let parts: Vec<&str> = filename.split('_').collect();
            if parts.len() < 4 {
                continue;
            }

            // Find the discipline name: parts[0] is the number prefix, then name parts
            // until we hit a numeric-only part (the steps count)
            let mut name_parts = Vec::new();
            for &part in &parts[1..] {
                if part.chars().all(|c| c.is_ascii_digit()) {
                    break;
                }
                name_parts.push(part);
            }
            if name_parts.is_empty() {
                continue;
            }

            let discipline_key = format!("{}_{}", parts[0], name_parts.join("_"));
            let discipline_name = name_parts.join("_");

            // The base36 timestamp is the last part — use it for sorting
            let timestamp = (*parts.last().unwrap()).to_owned();

            by_discipline
                .entry(discipline_key)
                .or_default()
                .push((timestamp, path.clone()));

            // Store the discipline name for later
            if !images
                .iter()
                .any(|(sid, n, _)| *sid == stack_id && *n == discipline_name)
            {
                images.push((stack_id, discipline_name, PathBuf::new()));
            }
        }

        // For each discipline, pick the PNG with the highest (latest) base36 timestamp
        for (key, mut candidates) in by_discipline {
            candidates.sort_by(|a, b| a.0.cmp(&b.0));
            let latest_path = candidates.last().unwrap().1.clone();

            let parts: Vec<&str> = key.split('_').collect();
            let name_parts = &parts[1..];
            let discipline_name = name_parts.join("_");

            // Update the entry with the actual path
            if let Some(entry) = images
                .iter_mut()
                .find(|(sid, n, _)| *sid == stack_id && *n == discipline_name)
            {
                entry.2 = latest_path;
            }
        }
    }

    // Remove entries with empty paths (shouldn't happen, but be safe)
    images.retain(|(_, _, p)| !p.as_os_str().is_empty());

    // Sort for deterministic output
    images.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    // Generate Rust source
    let mut code = String::from("pub const DISCIPLINE_IMAGES: &[(u8, &str, &[u8])] = &[\n");
    for (stack_id, name, path) in &images {
        let abs_path = path.to_string_lossy();
        code.push_str(&format!(
            "    ({stack_id}, \"{name}\", include_bytes!(\"{abs_path}\")),\n"
        ));
        // Tell cargo to rebuild if this file changes
        println!("cargo:rerun-if-changed={abs_path}");
    }
    code.push_str("];\n");

    std::fs::write(out_dir.join("discipline_images.rs"), code).unwrap();

    // Rebuild if images directories change
    for &(_, stack_dir) in stacks {
        let images_dir = manifest_dir
            .join("src/defaults/disciplines")
            .join(stack_dir)
            .join("images");
        println!("cargo:rerun-if-changed={}", images_dir.to_string_lossy());
    }
}
