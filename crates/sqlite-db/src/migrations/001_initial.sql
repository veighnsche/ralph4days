CREATE TABLE metadata (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  schema_version TEXT NOT NULL DEFAULT '1.0',
  project_title TEXT NOT NULL,
  project_description TEXT,
  project_created TEXT
) STRICT;

CREATE TABLE features (
  name TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,
  description TEXT,
  created TEXT,
  knowledge_paths TEXT DEFAULT '[]',
  context_files TEXT DEFAULT '[]',
  architecture TEXT,
  boundaries TEXT,
  learnings TEXT DEFAULT '[]',
  dependencies TEXT DEFAULT '[]'
) STRICT;

CREATE TABLE disciplines (
  name TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,
  icon TEXT NOT NULL,
  color TEXT NOT NULL,
  system_prompt TEXT,
  skills TEXT DEFAULT '[]',
  conventions TEXT,
  mcp_servers TEXT DEFAULT '[]',
  stack_id INTEGER DEFAULT NULL,
  image_path TEXT DEFAULT NULL,
  crops TEXT DEFAULT NULL,
  description TEXT DEFAULT NULL,
  image_prompt TEXT DEFAULT NULL
) STRICT;

CREATE TABLE tasks (
  id INTEGER PRIMARY KEY,
  feature TEXT NOT NULL REFERENCES features(name) ON DELETE RESTRICT,
  discipline TEXT NOT NULL REFERENCES disciplines(name) ON DELETE RESTRICT,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  priority TEXT,
  tags TEXT DEFAULT '[]',
  depends_on TEXT DEFAULT '[]',
  blocked_by TEXT,
  created TEXT,
  updated TEXT,
  completed TEXT,
  acceptance_criteria TEXT DEFAULT '[]',
  context_files TEXT DEFAULT '[]',
  output_artifacts TEXT DEFAULT '[]',
  hints TEXT,
  estimated_turns INTEGER,
  provenance TEXT,
  pseudocode TEXT DEFAULT NULL,
  enriched_at TEXT DEFAULT NULL
) STRICT;

CREATE TABLE task_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  session_id TEXT,
  author TEXT NOT NULL,

  signal_verb TEXT CHECK(signal_verb IN ('done','partial','stuck','ask','flag','learned','suggest','blocked') OR signal_verb IS NULL),
  signal_payload TEXT,
  signal_answered TEXT,

  body TEXT NOT NULL,
  created TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

CREATE TABLE recipe_configs (
  name TEXT PRIMARY KEY,
  base_recipe TEXT NOT NULL,
  section_order TEXT NOT NULL DEFAULT '[]',
  sections TEXT NOT NULL DEFAULT '{}',
  created TEXT,
  updated TEXT
) STRICT;

CREATE INDEX idx_tasks_feature ON tasks(feature);
CREATE INDEX idx_tasks_discipline ON tasks(discipline);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_comments_task ON task_comments(task_id);
CREATE INDEX idx_comments_session ON task_comments(session_id);
CREATE INDEX idx_comments_verb ON task_comments(signal_verb);
