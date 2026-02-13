-- Ralph Database Schema - Fully Normalized
-- No JSON, proper INTEGER PKs/FKs, comprehensive constraints

PRAGMA foreign_keys = ON;

-- Singleton metadata table
CREATE TABLE metadata (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  schema_version TEXT NOT NULL DEFAULT '1.0',
  project_title TEXT NOT NULL,
  project_description TEXT,
  project_created TEXT
) STRICT;

-- Features
CREATE TABLE features (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,
  description TEXT,
  architecture TEXT,
  boundaries TEXT,
  status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active','archived')),
  created TEXT
) STRICT;

CREATE TABLE feature_knowledge_paths (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
  path TEXT NOT NULL
) STRICT;

CREATE TABLE feature_context_files (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
  file_path TEXT NOT NULL
) STRICT;

CREATE TABLE feature_learnings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
  learning TEXT NOT NULL
) STRICT;

CREATE TABLE feature_dependencies (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
  depends_on_feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
  CHECK (feature_id != depends_on_feature_id),
  UNIQUE(feature_id, depends_on_feature_id)
) STRICT;

-- Disciplines
CREATE TABLE disciplines (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE,
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,
  icon TEXT NOT NULL,
  color TEXT NOT NULL,
  description TEXT,
  system_prompt TEXT,
  agent TEXT CHECK(agent IN ('claude','codex') OR agent IS NULL),
  model TEXT,
  effort TEXT CHECK(effort IN ('low','medium','high') OR effort IS NULL),
  thinking INTEGER CHECK(thinking IN (0,1) OR thinking IS NULL),
  conventions TEXT,
  stack_id INTEGER,
  image_path TEXT,
  crops TEXT,
  image_prompt TEXT
) STRICT;

CREATE TABLE discipline_skills (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  discipline_id INTEGER NOT NULL REFERENCES disciplines(id) ON DELETE CASCADE,
  skill TEXT NOT NULL
) STRICT;

CREATE TABLE discipline_mcp_servers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  discipline_id INTEGER NOT NULL REFERENCES disciplines(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  command TEXT NOT NULL
) STRICT;

CREATE TABLE discipline_mcp_server_args (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  server_id INTEGER NOT NULL REFERENCES discipline_mcp_servers(id) ON DELETE CASCADE,
  arg TEXT NOT NULL,
  arg_order INTEGER NOT NULL
) STRICT;

CREATE TABLE discipline_mcp_server_env (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  server_id INTEGER NOT NULL REFERENCES discipline_mcp_servers(id) ON DELETE CASCADE,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  UNIQUE(server_id, key)
) STRICT;

-- Tasks
CREATE TABLE tasks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE RESTRICT,
  discipline_id INTEGER NOT NULL REFERENCES disciplines(id) ON DELETE RESTRICT,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('draft','pending','in_progress','done','blocked','skipped','needs_input','failed')),
  priority TEXT CHECK(priority IN ('low','medium','high','critical') OR priority IS NULL),
  hints TEXT,
  estimated_turns INTEGER,
  provenance TEXT CHECK(provenance IN ('agent','human','system') OR provenance IS NULL),
  pseudocode TEXT,
  enriched_at TEXT,
  created TEXT,
  updated TEXT,
  completed TEXT
) STRICT;

CREATE TABLE task_tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  tag TEXT NOT NULL
) STRICT;

CREATE TABLE task_dependencies (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  depends_on_task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  CHECK (task_id != depends_on_task_id),
  UNIQUE(task_id, depends_on_task_id)
) STRICT;

CREATE TABLE task_acceptance_criteria (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  criterion TEXT NOT NULL,
  criterion_order INTEGER NOT NULL
) STRICT;

CREATE TABLE task_context_files (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  file_path TEXT NOT NULL
) STRICT;

CREATE TABLE task_output_artifacts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  artifact_path TEXT NOT NULL
) STRICT;

-- Agent sessions (agent runs, human starts, braindumps, reviews)
CREATE TABLE agent_sessions (
  id TEXT PRIMARY KEY, -- e.g. RALPH_SESSION_ID
  session_number INTEGER NOT NULL UNIQUE CHECK(session_number > 0),
  kind TEXT NOT NULL CHECK(kind IN ('task_execution','human_braindump','manual','review')),
  started_by TEXT NOT NULL CHECK(started_by IN ('agent','human','system')),
  task_id INTEGER REFERENCES tasks(id) ON DELETE SET NULL,
  agent TEXT, -- claude, codex, etc.
  model TEXT,
  launch_command TEXT,
  post_start_preamble TEXT, -- newline-delimited steps; interpreted by agent-specific runner
  init_prompt TEXT,
  started TEXT NOT NULL DEFAULT (datetime('now')),
  ended TEXT,
  exit_code INTEGER,
  closing_verb TEXT CHECK(closing_verb IN ('done','partial','stuck') OR closing_verb IS NULL),
  status TEXT NOT NULL DEFAULT 'running' CHECK(status IN ('running','finished','crashed','timed_out','cancelled')),
  prompt_hash TEXT,
  output_bytes INTEGER CHECK(output_bytes >= 0 OR output_bytes IS NULL),
  error_text TEXT,
  CHECK((status = 'running' AND ended IS NULL) OR (status != 'running'))
) STRICT;

-- Task signals (flat columns, no payload JSON)
CREATE TABLE task_signals (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  session_id TEXT NOT NULL,

  verb TEXT NOT NULL CHECK(verb IN ('done','partial','stuck','ask','flag','learned','suggest','blocked')),

  -- Verb payload fields (flat columns)
  text TEXT,
  summary TEXT,
  remaining TEXT,
  reason TEXT,
  question TEXT,
  what TEXT,
  "on" TEXT,

  -- Metadata
  blocking INTEGER CHECK(blocking IN (0,1) OR blocking IS NULL),
  severity TEXT CHECK(severity IN ('info','warning','blocking') OR severity IS NULL),
  category TEXT CHECK(category IN ('bug','stale','contradiction','ambiguity','overlap','performance','security','incomplete_prior') OR category IS NULL),
  kind TEXT,
  scope TEXT CHECK(scope IN ('project','feature','task') OR scope IS NULL),

  -- Additional details
  preferred TEXT,
  options TEXT, -- newline-separated for 'ask' verb
  rationale TEXT,
  why TEXT,
  detail TEXT,

  -- Answer
  answer TEXT,

  created TEXT NOT NULL DEFAULT (datetime('now')),

  -- Verb-specific shape (required + disallowed fields)
  CHECK(
    (verb = 'done' AND summary IS NOT NULL AND remaining IS NULL AND reason IS NULL AND question IS NULL AND what IS NULL AND text IS NULL AND "on" IS NULL) OR
    (verb = 'partial' AND summary IS NOT NULL AND remaining IS NOT NULL AND reason IS NULL AND question IS NULL AND what IS NULL AND text IS NULL AND "on" IS NULL) OR
    (verb = 'stuck' AND reason IS NOT NULL AND summary IS NULL AND remaining IS NULL AND question IS NULL AND what IS NULL AND text IS NULL AND "on" IS NULL) OR
    (verb = 'ask' AND question IS NOT NULL AND blocking IS NOT NULL AND summary IS NULL AND remaining IS NULL AND reason IS NULL AND what IS NULL AND text IS NULL AND "on" IS NULL) OR
    (verb = 'flag' AND what IS NOT NULL AND severity IS NOT NULL AND category IS NOT NULL AND summary IS NULL AND remaining IS NULL AND reason IS NULL AND question IS NULL AND text IS NULL AND "on" IS NULL) OR
    (verb = 'learned' AND text IS NOT NULL AND kind IS NOT NULL AND summary IS NULL AND remaining IS NULL AND reason IS NULL AND question IS NULL AND what IS NULL AND "on" IS NULL) OR
    (verb = 'suggest' AND what IS NOT NULL AND kind IS NOT NULL AND why IS NOT NULL AND summary IS NULL AND remaining IS NULL AND reason IS NULL AND question IS NULL AND text IS NULL AND "on" IS NULL) OR
    (verb = 'blocked' AND "on" IS NOT NULL AND kind IS NOT NULL AND summary IS NULL AND remaining IS NULL AND reason IS NULL AND question IS NULL AND what IS NULL AND text IS NULL)
  ),

  -- Field-level enums by verb
  CHECK(
    (verb = 'flag' AND severity IN ('info','warning','blocking') AND category IN ('bug','stale','contradiction','ambiguity','overlap','performance','security','incomplete_prior')) OR
    (verb != 'flag' AND severity IS NULL AND category IS NULL)
  ),
  CHECK(
    (verb = 'learned' AND kind IN ('discovery','decision','convention')) OR
    (verb = 'suggest' AND kind IN ('new_task','split','refactor','alternative','deprecate')) OR
    (verb = 'blocked' AND kind IN ('upstream_task','external')) OR
    (verb NOT IN ('learned','suggest','blocked') AND kind IS NULL)
  ),
  CHECK(
    (verb = 'learned' AND (scope IN ('project','feature','task') OR scope IS NULL)) OR
    (verb != 'learned' AND scope IS NULL)
  ),
  CHECK((verb = 'ask') OR (blocking IS NULL AND options IS NULL AND preferred IS NULL AND answer IS NULL)),
  CHECK((verb = 'learned') OR (rationale IS NULL)),
  CHECK((verb = 'suggest') OR (why IS NULL)),
  CHECK((verb = 'blocked') OR (detail IS NULL)),

  FOREIGN KEY (session_id) REFERENCES agent_sessions(id) ON DELETE CASCADE
) STRICT;

-- Comments under task signals (flat, non-threaded)
CREATE TABLE task_signal_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  signal_id INTEGER NOT NULL REFERENCES task_signals(id) ON DELETE CASCADE,
  session_id TEXT REFERENCES agent_sessions(id) ON DELETE SET NULL,
  author_type TEXT NOT NULL CHECK(author_type IN ('human','agent','system')),
  body TEXT NOT NULL,
  created TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

-- Feature Comments (knowledge layer for features)
CREATE TABLE feature_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  feature_id INTEGER NOT NULL REFERENCES features(id) ON DELETE CASCADE,
  category TEXT NOT NULL CHECK(category IN ('architecture','boundary','learning','convention','dependency','design-decision','gotcha')),
  discipline_id INTEGER REFERENCES disciplines(id) ON DELETE SET NULL,
  agent_task_id INTEGER,
  body TEXT NOT NULL,
  summary TEXT,
  reason TEXT,
  source_iteration INTEGER,
  created TEXT,
  updated TEXT
) STRICT;

-- Comment Embeddings (vector search for feature comments)
CREATE TABLE comment_embeddings (
  comment_id INTEGER PRIMARY KEY REFERENCES feature_comments(id) ON DELETE CASCADE,
  embedding BLOB NOT NULL,
  embedding_model TEXT NOT NULL,
  embedding_hash TEXT NOT NULL
) STRICT;

-- Prompt builder configs
CREATE TABLE prompt_builder_configs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  base_prompt TEXT NOT NULL,
  section_order TEXT NOT NULL,
  sections TEXT NOT NULL,
  created TEXT,
  updated TEXT
) STRICT;

-- Indexes for performance
CREATE INDEX idx_features_name ON features(name);
CREATE INDEX idx_disciplines_name ON disciplines(name);

CREATE INDEX idx_tasks_feature ON tasks(feature_id);
CREATE INDEX idx_tasks_discipline ON tasks(discipline_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_priority ON tasks(priority);

CREATE INDEX idx_task_tags_task ON task_tags(task_id);
CREATE INDEX idx_task_deps_task ON task_dependencies(task_id);
CREATE INDEX idx_task_deps_depends ON task_dependencies(depends_on_task_id);

CREATE INDEX idx_agent_sessions_task ON agent_sessions(task_id);
CREATE INDEX idx_agent_sessions_status ON agent_sessions(status);
CREATE INDEX idx_agent_sessions_started ON agent_sessions(started);

CREATE INDEX idx_signals_task ON task_signals(task_id);
CREATE INDEX idx_signals_session ON task_signals(session_id);
CREATE INDEX idx_signals_verb ON task_signals(verb);
CREATE INDEX idx_signals_task_verb ON task_signals(task_id, verb);

CREATE INDEX idx_signal_comments_signal ON task_signal_comments(signal_id);
CREATE INDEX idx_signal_comments_session ON task_signal_comments(session_id);
CREATE INDEX idx_signal_comments_created ON task_signal_comments(created);

CREATE INDEX idx_feature_comments_feature ON feature_comments(feature_id);
CREATE INDEX idx_feature_comments_category ON feature_comments(category);
CREATE INDEX idx_feature_comments_discipline ON feature_comments(discipline_id);

CREATE INDEX idx_prompt_builder_configs_name ON prompt_builder_configs(name);

-- Auto-update timestamp trigger
CREATE TRIGGER update_task_timestamp
AFTER UPDATE ON tasks
FOR EACH ROW
WHEN NEW.updated IS NULL OR OLD.updated = NEW.updated
BEGIN
  UPDATE tasks SET updated = datetime('now') WHERE id = NEW.id;
END;
