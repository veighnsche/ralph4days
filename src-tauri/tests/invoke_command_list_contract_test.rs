use sha2::{Digest, Sha256};

const LIB_RS: &str = include_str!("../src/lib.rs");

// Update this intentionally when the frontend-facing invoke command surface changes.
const EXPECTED_INVOKE_COMMAND_LIST_SHA256: &str =
    "81f7ea0dfb290f18676ba900b328d28f356846e699444091e31bdbfdedde9903";

fn sha256_hex(text: &str) -> String {
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn extract_invoke_handler_list_source() -> &'static str {
    let start = ".invoke_handler(tauri::generate_handler![";
    let (_, after) = LIB_RS
        .split_once(start)
        .unwrap_or_else(|| panic!("Failed to find invoke handler start marker: {start}"));

    let (list, _) = after
        .split_once("])")
        .expect("Failed to find invoke handler end marker (expected `])` after start marker)");
    list
}

fn current_invoke_command_names_sorted() -> Vec<String> {
    let list_source = extract_invoke_handler_list_source();
    let mut names = Vec::new();

    for raw in list_source.lines() {
        // Allow inline comments in the command list without breaking the contract test.
        let raw = raw.split("//").next().unwrap_or(raw);
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }

        let line = line
            .strip_suffix(',')
            .unwrap_or_else(|| panic!("Command line missing trailing comma: {line}"));

        assert!(
            line.starts_with("commands::") || line.starts_with("crate::commands::"),
            "Unexpected invoke command list entry: {line}"
        );

        let command = line
            .rsplit("::")
            .next()
            .unwrap_or_else(|| panic!("Failed to parse command name from path: {line}"))
            .trim();
        names.push(command.to_owned());
    }

    names.sort();
    names
}

#[test]
fn invoke_command_list_is_stable() {
    let names = current_invoke_command_names_sorted();
    let text = format!("{}\n", names.join("\n"));
    let actual_hash = sha256_hex(&text);

    assert!(
        actual_hash == EXPECTED_INVOKE_COMMAND_LIST_SHA256,
        "Invoke command list changed.\n\
         expected sha256: {EXPECTED_INVOKE_COMMAND_LIST_SHA256}\n\
         actual sha256:   {actual_hash}\n\
         current list (sorted):\n\
         {text}"
    );
}
