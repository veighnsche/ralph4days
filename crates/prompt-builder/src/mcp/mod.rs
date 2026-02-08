pub mod helpers;
pub mod tools;

use crate::context::PromptContext;
use crate::output::McpScript;
use tools::McpTool;

/// Generate MCP bash script + config JSON from context and requested tools.
///
/// Returns a list of scripts to write to disk and a JSON string for the
/// `--mcp-config` flag. When `tools` is empty, returns no scripts and an
/// empty JSON object.
pub fn generate(ctx: &PromptContext, tools: &[McpTool]) -> (Vec<McpScript>, String) {
    if tools.is_empty() {
        return (vec![], "{}".to_owned());
    }

    let script_content = generate_script(ctx, tools);
    let filename = "ralph-mcp.sh".to_owned();
    let scripts = vec![McpScript {
        filename: filename.clone(),
        content: script_content,
    }];

    let config = generate_config(ctx, &filename);
    (scripts, config)
}

/// Build the complete bash MCP server script.
fn generate_script(ctx: &PromptContext, tools: &[McpTool]) -> String {
    let mut s = String::with_capacity(8192);

    // Header: shebang + init handler + notification handler
    s.push_str(helpers::mcp_header());

    // Environment variables for DB path and project path
    let escaped_db = bash_escape(&ctx.db_path);
    let escaped_project = bash_escape(&ctx.project_path);
    s.push_str(&format!("RALPH_DB='{escaped_db}'\n"));
    s.push_str(&format!("PROJECT_PATH='{escaped_project}'\n\n"));

    // Helper functions
    s.push_str(helpers::id_extractor());
    s.push('\n');

    // Main read loop
    s.push_str("# Main JSON-RPC loop\n");
    s.push_str("while IFS= read -r line; do\n");
    s.push_str("    # Skip empty lines\n");
    s.push_str("    [ -z \"$line\" ] && continue\n\n");
    s.push_str("    id=$(extract_id \"$line\")\n\n");

    // Method dispatch
    s.push_str("    # Dispatch by method\n");
    s.push_str("    if echo \"$line\" | grep -q '\"method\"[[:space:]]*:[[:space:]]*\"initialize\"'; then\n");
    s.push_str("        handle_initialize \"$id\"\n\n");

    // Notifications (no response needed)
    s.push_str("    elif echo \"$line\" | grep -q '\"method\"[[:space:]]*:[[:space:]]*\"notifications/'; then\n");
    s.push_str("        handle_notification\n\n");

    // tools/list handler
    s.push_str("    elif echo \"$line\" | grep -q '\"method\"[[:space:]]*:[[:space:]]*\"tools/list\"'; then\n");
    s.push_str(&generate_tools_list(tools));

    // tools/call handler
    s.push_str("    elif echo \"$line\" | grep -q '\"method\"[[:space:]]*:[[:space:]]*\"tools/call\"'; then\n");
    s.push_str(&generate_tools_call(tools));

    // resources/list handler (empty)
    s.push_str("    elif echo \"$line\" | grep -q '\"method\"[[:space:]]*:[[:space:]]*\"resources/list\"'; then\n");
    s.push_str("        printf '{\"jsonrpc\":\"2.0\",\"id\":%s,\"result\":{\"resources\":[]}}\\n' \"$id\"\n\n");

    // Unknown method — return method not found
    s.push_str("    else\n");
    s.push_str("        printf '{\"jsonrpc\":\"2.0\",\"id\":%s,\"error\":{\"code\":-32601,\"message\":\"Method not found\"}}\\n' \"$id\"\n");
    s.push_str("    fi\n\n");

    s.push_str("done\n");
    s
}

/// Generate the tools/list response block for the given tools.
fn generate_tools_list(tools: &[McpTool]) -> String {
    let mut s = String::new();
    s.push_str("        cat <<'EOJSON'\n");
    s.push_str(&format!(
        "{{\"jsonrpc\":\"2.0\",\"id\":IDPLACEHOLDER,\"result\":{{\"tools\":[{}]}}}}\n",
        tools
            .iter()
            .map(|t| {
                format!(
                    "{{\"name\":\"{}\",\"description\":\"{}\",\"inputSchema\":{}}}",
                    t.tool_name(),
                    t.tool_description(),
                    t.tool_schema()
                )
            })
            .collect::<Vec<_>>()
            .join(",")
    ));
    s.push_str("EOJSON\n");

    // We need to replace the placeholder with the actual id. Heredocs can't
    // interpolate variables, so we pipe through sed.
    // Actually, let's use printf instead for cleaner output.
    // Rewrite: use printf with the JSON embedded.
    s.clear();

    // Build the tools array as a static string
    let tools_json: String = tools
        .iter()
        .map(|t| {
            format!(
                "{{\"name\":\"{}\",\"description\":\"{}\",\"inputSchema\":{}}}",
                t.tool_name(),
                t.tool_description(),
                t.tool_schema()
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    s.push_str(&format!(
        "        printf '{{\"jsonrpc\":\"2.0\",\"id\":%s,\"result\":{{\"tools\":[{tools_json}]}}}}\\n' \"$id\"\n\n"
    ));
    s
}

/// Generate the tools/call dispatch block with if/elif chain.
fn generate_tools_call(tools: &[McpTool]) -> String {
    let mut s = String::new();

    // Extract the tool name from the request
    s.push_str("        tool_name=$(echo \"$line\" | sed -n 's/.*\"name\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p')\n\n");

    for (i, tool) in tools.iter().enumerate() {
        if i == 0 {
            s.push_str(&format!(
                "        if [ \"$tool_name\" = \"{}\" ]; then\n",
                tool.tool_name()
            ));
        } else {
            s.push_str(&format!(
                "        elif [ \"$tool_name\" = \"{}\" ]; then\n",
                tool.tool_name()
            ));
        }
        s.push_str(tool.tool_handler());
    }

    // Unknown tool
    s.push_str("        else\n");
    s.push_str("            e_msg=$(json_escape \"Unknown tool: $tool_name\")\n");
    s.push_str("            printf '{\"jsonrpc\":\"2.0\",\"id\":%s,\"error\":{\"code\":-32601,\"message\":\"%s\"}}\\n' \"$id\" \"$e_msg\"\n");
    s.push_str("        fi\n\n");

    s
}

/// Generate the MCP config JSON for `--mcp-config`.
///
/// Includes the ralph MCP server and any discipline-specific MCP servers
/// from the target task's discipline.
fn generate_config(ctx: &PromptContext, filename: &str) -> String {
    let script_path = format!("{}/{}", ctx.script_dir, filename);
    let escaped_path = json_escape(&script_path);

    let mut servers = Vec::new();

    // Ralph's own MCP server
    servers.push(format!(
        "\"ralph\":{{\"command\":\"{escaped_path}\",\"args\":[]}}"
    ));

    // Discipline-specific MCP servers (for TaskExecution prompts)
    if let Some(discipline) = ctx.target_task_discipline() {
        for mcp in &discipline.mcp_servers {
            let name = json_escape(&mcp.name);
            let command = json_escape(&mcp.command);
            let args: Vec<String> = mcp
                .args
                .iter()
                .map(|a| format!("\"{}\"", json_escape(a)))
                .collect();
            let args_str = args.join(",");

            let mut server =
                format!("\"{name}\":{{\"command\":\"{command}\",\"args\":[{args_str}]");

            if !mcp.env.is_empty() {
                let env_entries: Vec<String> = mcp
                    .env
                    .iter()
                    .map(|(k, v)| format!("\"{}\":\"{}\"", json_escape(k), json_escape(v)))
                    .collect();
                server.push_str(&format!(",\"env\":{{{}}}", env_entries.join(",")));
            }

            server.push('}');
            servers.push(server);
        }
    }

    format!("{{\"mcpServers\":{{{}}}}}", servers.join(","))
}

/// Escape a string for embedding in a single-quoted bash string.
/// Single quotes cannot appear inside single-quoted strings in bash, so we
/// end the string, add an escaped single quote, and restart.
fn bash_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Escape a string for embedding in a JSON string value.
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::test_context;

    #[test]
    fn empty_tools_returns_empty() {
        let ctx = test_context();
        let (scripts, config) = generate(&ctx, &[]);
        assert!(scripts.is_empty());
        assert_eq!(config, "{}");
    }

    #[test]
    fn single_tool_generates_script_and_config() {
        let ctx = test_context();
        let (scripts, config) = generate(&ctx, &[McpTool::ListTasks]);
        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0].filename, "ralph-mcp.sh");
        assert!(scripts[0].content.contains("#!/usr/bin/env bash"));
        assert!(scripts[0].content.contains("list_tasks"));
        assert!(scripts[0].content.contains("RALPH_DB="));
        assert!(config.contains("\"ralph\""));
        assert!(config.contains("ralph-mcp.sh"));
    }

    #[test]
    fn multiple_tools_all_present_in_script() {
        let ctx = test_context();
        let tools = vec![
            McpTool::CreateFeature,
            McpTool::CreateTask,
            McpTool::SetTaskStatus,
        ];
        let (scripts, _config) = generate(&ctx, &tools);
        let content = &scripts[0].content;
        assert!(content.contains("create_feature"));
        assert!(content.contains("create_task"));
        assert!(content.contains("set_task_status"));
    }

    #[test]
    fn config_includes_discipline_mcp_servers() {
        use sqlite_db::{Discipline, McpServerConfig};
        use std::collections::HashMap;

        let mut ctx = test_context();
        ctx.target_task_id = Some(1);
        ctx.tasks = vec![sqlite_db::Task {
            id: 1,
            feature: "auth".to_owned(),
            discipline: "frontend".to_owned(),
            title: "Build login".to_owned(),
            description: None,
            status: sqlite_db::TaskStatus::Pending,
            inferred_status: sqlite_db::InferredTaskStatus::Ready,
            priority: Some(sqlite_db::Priority::Medium),
            tags: vec![],
            depends_on: vec![],
            blocked_by: None,
            created: None,
            updated: None,
            completed: None,
            acceptance_criteria: vec![],
            context_files: vec![],
            output_artifacts: vec![],
            hints: None,
            estimated_turns: None,
            provenance: None,
            comments: vec![],
            feature_display_name: "Auth".to_owned(),
            feature_acronym: "AU".to_owned(),
            discipline_display_name: "Frontend".to_owned(),
            discipline_acronym: "FE".to_owned(),
            discipline_icon: "code".to_owned(),
            discipline_color: "blue".to_owned(),
        }];
        let mut env = HashMap::new();
        env.insert("NODE_ENV".to_owned(), "development".to_owned());
        ctx.disciplines = vec![Discipline {
            name: "frontend".to_owned(),
            display_name: "Frontend".to_owned(),
            icon: "code".to_owned(),
            color: "blue".to_owned(),
            acronym: "FE".to_owned(),
            system_prompt: None,
            skills: vec![],
            conventions: None,
            mcp_servers: vec![McpServerConfig {
                name: "browser-tools".to_owned(),
                command: "npx".to_owned(),
                args: vec!["@anthropic/browser-tools".to_owned()],
                env,
            }],
        }];

        let (_scripts, config) = generate(&ctx, &[McpTool::SetTaskStatus]);
        assert!(config.contains("\"browser-tools\""));
        assert!(config.contains("npx"));
        assert!(config.contains("@anthropic/browser-tools"));
        assert!(config.contains("NODE_ENV"));
    }

    #[test]
    fn bash_escape_handles_single_quotes() {
        assert_eq!(bash_escape("it's"), "it'\\''s");
        assert_eq!(bash_escape("no quotes"), "no quotes");
    }

    #[test]
    fn json_escape_handles_special_chars() {
        assert_eq!(json_escape("a\"b"), "a\\\"b");
        assert_eq!(json_escape("a\\b"), "a\\\\b");
    }
}
