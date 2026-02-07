/// The set of MCP tools Ralph can expose to Claude via generated bash scripts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McpTool {
    CreateFeature,
    CreateDiscipline,
    CreateTask,
    UpdateTask,
    SetTaskStatus,
    ListFeatures,
    ListDisciplines,
    ListTasks,
    UpdateFeature,
    UpdateDiscipline,
    AppendLearning,
    AddContextFile,
}

impl McpTool {
    /// Snake_case name used in MCP tools/list and tools/call.
    pub fn tool_name(&self) -> &'static str {
        match self {
            Self::CreateFeature => "create_feature",
            Self::CreateDiscipline => "create_discipline",
            Self::CreateTask => "create_task",
            Self::UpdateTask => "update_task",
            Self::SetTaskStatus => "set_task_status",
            Self::ListFeatures => "list_features",
            Self::ListDisciplines => "list_disciplines",
            Self::ListTasks => "list_tasks",
            Self::UpdateFeature => "update_feature",
            Self::UpdateDiscipline => "update_discipline",
            Self::AppendLearning => "append_learning",
            Self::AddContextFile => "add_context_file",
        }
    }

    /// Human-readable description for tool discovery.
    pub fn tool_description(&self) -> &'static str {
        match self {
            Self::CreateFeature => "Create a new feature in the project database",
            Self::CreateDiscipline => "Create a new discipline in the project database",
            Self::CreateTask => "Create a new task in the project database",
            Self::UpdateTask => "Update an existing task's title, description, or priority",
            Self::SetTaskStatus => {
                "Set the status of a task (pending, in_progress, done, blocked, skipped)"
            }
            Self::ListFeatures => "List all features in the project",
            Self::ListDisciplines => "List all disciplines in the project",
            Self::ListTasks => "List all tasks in the project",
            Self::UpdateFeature => "Update an existing feature's display name or description",
            Self::UpdateDiscipline => {
                "Update an existing discipline's system prompt or conventions"
            }
            Self::AppendLearning => "Append a learning entry to the project's learnings.txt",
            Self::AddContextFile => "Add a context file path to a task",
        }
    }

    /// JSON schema string for the tool's `inputSchema` in MCP tools/list.
    pub fn tool_schema(&self) -> &'static str {
        match self {
            Self::CreateFeature => {
                r#"{"type":"object","properties":{"name":{"type":"string","description":"Unique snake_case feature name"},"display_name":{"type":"string","description":"Human-readable feature name"},"description":{"type":"string","description":"Feature description"}},"required":["name","display_name"]}"#
            }

            Self::CreateDiscipline => {
                r#"{"type":"object","properties":{"name":{"type":"string","description":"Unique snake_case discipline name"},"display_name":{"type":"string","description":"Human-readable discipline name"},"icon":{"type":"string","description":"Lucide icon name"},"color":{"type":"string","description":"Tailwind color class"}},"required":["name","display_name","icon","color"]}"#
            }

            Self::CreateTask => {
                r#"{"type":"object","properties":{"feature":{"type":"string","description":"Feature name this task belongs to"},"discipline":{"type":"string","description":"Discipline name for this task"},"title":{"type":"string","description":"Task title"},"description":{"type":"string","description":"Detailed task description"},"priority":{"type":"string","description":"Priority: critical, high, medium, low"},"acceptance_criteria":{"type":"string","description":"Semicolon-separated acceptance criteria"}},"required":["feature","discipline","title"]}"#
            }

            Self::UpdateTask => {
                r#"{"type":"object","properties":{"id":{"type":"number","description":"Task ID to update"},"title":{"type":"string","description":"New task title"},"description":{"type":"string","description":"New task description"},"priority":{"type":"string","description":"New priority: critical, high, medium, low"}},"required":["id"]}"#
            }

            Self::SetTaskStatus => {
                r#"{"type":"object","properties":{"id":{"type":"number","description":"Task ID"},"status":{"type":"string","enum":["pending","in_progress","done","blocked","skipped"],"description":"New task status"}},"required":["id","status"]}"#
            }

            Self::ListFeatures => r#"{"type":"object","properties":{}}"#,

            Self::ListDisciplines => r#"{"type":"object","properties":{}}"#,

            Self::ListTasks => r#"{"type":"object","properties":{}}"#,

            Self::UpdateFeature => {
                r#"{"type":"object","properties":{"name":{"type":"string","description":"Feature name to update"},"display_name":{"type":"string","description":"New display name"},"description":{"type":"string","description":"New description"}},"required":["name"]}"#
            }

            Self::UpdateDiscipline => {
                r#"{"type":"object","properties":{"name":{"type":"string","description":"Discipline name to update"},"system_prompt":{"type":"string","description":"New system prompt"},"conventions":{"type":"string","description":"New conventions text"}},"required":["name"]}"#
            }

            Self::AppendLearning => {
                r#"{"type":"object","properties":{"text":{"type":"string","description":"Learning text to append"}},"required":["text"]}"#
            }

            Self::AddContextFile => {
                r#"{"type":"object","properties":{"task_id":{"type":"number","description":"Task ID to add context file to"},"file_path":{"type":"string","description":"Relative file path to add as context"}},"required":["task_id","file_path"]}"#
            }
        }
    }

    /// Bash snippet for the tools/call handler. Assumes `$line` contains the
    /// JSON-RPC request and `$id` holds the extracted request ID.
    /// Uses `$RALPH_DB` and `$PROJECT_PATH` environment variables set at
    /// script top.
    pub fn tool_handler(&self) -> &'static str {
        match self {
            Self::CreateFeature => {
                r#"
        name=$(echo "$line" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        display_name=$(echo "$line" | sed -n 's/.*"display_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        description=$(echo "$line" | sed -n 's/.*"description"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_name=$(json_escape "$name")
        e_display=$(json_escape "$display_name")
        e_desc=$(json_escape "$description")
        sqlite3 "$RALPH_DB" "INSERT INTO features (name, display_name, description, created) VALUES ('${e_name}', '${e_display}', '${e_desc}', datetime('now'));"
        result="Created feature: $name"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::CreateDiscipline => {
                r#"
        name=$(echo "$line" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        display_name=$(echo "$line" | sed -n 's/.*"display_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        icon=$(echo "$line" | sed -n 's/.*"icon"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        color=$(echo "$line" | sed -n 's/.*"color"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_name=$(json_escape "$name")
        e_display=$(json_escape "$display_name")
        e_icon=$(json_escape "$icon")
        e_color=$(json_escape "$color")
        sqlite3 "$RALPH_DB" "INSERT INTO disciplines (name, display_name, icon, color) VALUES ('${e_name}', '${e_display}', '${e_icon}', '${e_color}');"
        result="Created discipline: $name"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::CreateTask => {
                r#"
        feature=$(echo "$line" | sed -n 's/.*"feature"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        discipline=$(echo "$line" | sed -n 's/.*"discipline"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        title=$(echo "$line" | sed -n 's/.*"title"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        description=$(echo "$line" | sed -n 's/.*"description"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        priority=$(echo "$line" | sed -n 's/.*"priority"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        acceptance_criteria=$(echo "$line" | sed -n 's/.*"acceptance_criteria"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_feature=$(json_escape "$feature")
        e_discipline=$(json_escape "$discipline")
        e_title=$(json_escape "$title")
        e_desc=$(json_escape "$description")
        e_priority=$(json_escape "${priority:-medium}")
        e_ac=$(json_escape "$acceptance_criteria")
        new_id=$(sqlite3 "$RALPH_DB" "SELECT COALESCE(MAX(id),0)+1 FROM tasks;")
        sqlite3 "$RALPH_DB" "INSERT INTO tasks (id, feature, discipline, title, description, status, priority, acceptance_criteria, created) VALUES (${new_id}, '${e_feature}', '${e_discipline}', '${e_title}', '${e_desc}', 'pending', '${e_priority}', '${e_ac}', datetime('now'));"
        result="Created task #${new_id}: $title"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::UpdateTask => {
                r#"
        task_id=$(echo "$line" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p')
        title=$(echo "$line" | sed -n 's/.*"title"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        description=$(echo "$line" | sed -n 's/.*"description"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        priority=$(echo "$line" | sed -n 's/.*"priority"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        updates=""
        if [ -n "$title" ]; then
            e_title=$(json_escape "$title")
            updates="title='${e_title}'"
        fi
        if [ -n "$description" ]; then
            e_desc=$(json_escape "$description")
            [ -n "$updates" ] && updates="$updates, "
            updates="${updates}description='${e_desc}'"
        fi
        if [ -n "$priority" ]; then
            e_priority=$(json_escape "$priority")
            [ -n "$updates" ] && updates="$updates, "
            updates="${updates}priority='${e_priority}'"
        fi
        if [ -n "$updates" ]; then
            sqlite3 "$RALPH_DB" "UPDATE tasks SET ${updates} WHERE id=${task_id};"
        fi
        result="Updated task #${task_id}"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::SetTaskStatus => {
                r#"
        task_id=$(echo "$line" | sed -n 's/.*"id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p')
        status=$(echo "$line" | sed -n 's/.*"status"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_status=$(json_escape "$status")
        if [ "$status" = "done" ]; then
            sqlite3 "$RALPH_DB" "UPDATE tasks SET status='${e_status}', completed=datetime('now') WHERE id=${task_id};"
        else
            sqlite3 "$RALPH_DB" "UPDATE tasks SET status='${e_status}' WHERE id=${task_id};"
        fi
        result="Task #${task_id} status set to: $status"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::ListFeatures => {
                r#"
        rows=$(sqlite3 -separator '|' "$RALPH_DB" "SELECT name, display_name, description FROM features ORDER BY name;")
        result="name|display_name|description"$'\n'"$rows"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::ListDisciplines => {
                r#"
        rows=$(sqlite3 -separator '|' "$RALPH_DB" "SELECT name, display_name, icon, color FROM disciplines ORDER BY name;")
        result="name|display_name|icon|color"$'\n'"$rows"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::ListTasks => {
                r#"
        rows=$(sqlite3 -separator '|' "$RALPH_DB" "SELECT id, feature, discipline, title, status, priority FROM tasks ORDER BY id;")
        result="id|feature|discipline|title|status|priority"$'\n'"$rows"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::UpdateFeature => {
                r#"
        name=$(echo "$line" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        display_name=$(echo "$line" | sed -n 's/.*"display_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        description=$(echo "$line" | sed -n 's/.*"description"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_name=$(json_escape "$name")
        updates=""
        if [ -n "$display_name" ]; then
            e_display=$(json_escape "$display_name")
            updates="display_name='${e_display}'"
        fi
        if [ -n "$description" ]; then
            e_desc=$(json_escape "$description")
            [ -n "$updates" ] && updates="$updates, "
            updates="${updates}description='${e_desc}'"
        fi
        if [ -n "$updates" ]; then
            sqlite3 "$RALPH_DB" "UPDATE features SET ${updates} WHERE name='${e_name}';"
        fi
        result="Updated feature: $name"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::UpdateDiscipline => {
                r#"
        name=$(echo "$line" | sed -n 's/.*"name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        system_prompt=$(echo "$line" | sed -n 's/.*"system_prompt"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        conventions=$(echo "$line" | sed -n 's/.*"conventions"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_name=$(json_escape "$name")
        updates=""
        if [ -n "$system_prompt" ]; then
            e_prompt=$(json_escape "$system_prompt")
            updates="system_prompt='${e_prompt}'"
        fi
        if [ -n "$conventions" ]; then
            e_conv=$(json_escape "$conventions")
            [ -n "$updates" ] && updates="$updates, "
            updates="${updates}conventions='${e_conv}'"
        fi
        if [ -n "$updates" ]; then
            sqlite3 "$RALPH_DB" "UPDATE disciplines SET ${updates} WHERE name='${e_name}';"
        fi
        result="Updated discipline: $name"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::AppendLearning => {
                r#"
        text=$(echo "$line" | sed -n 's/.*"text"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        echo "$text" >> "$PROJECT_PATH/.ralph/learnings.txt"
        result="Appended learning entry"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }

            Self::AddContextFile => {
                r#"
        task_id=$(echo "$line" | sed -n 's/.*"task_id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/p')
        file_path=$(echo "$line" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
        e_path=$(json_escape "$file_path")
        sqlite3 "$RALPH_DB" "UPDATE tasks SET context_files = CASE WHEN context_files IS NULL OR context_files = '' THEN '${e_path}' ELSE context_files || ';' || '${e_path}' END WHERE id=${task_id};"
        result="Added context file '$file_path' to task #${task_id}"
        e_result=$(json_escape "$result")
        printf '{"jsonrpc":"2.0","id":%s,"result":{"content":[{"type":"text","text":"%s"}]}}\n' "$id" "$e_result"
"#
            }
        }
    }
}
