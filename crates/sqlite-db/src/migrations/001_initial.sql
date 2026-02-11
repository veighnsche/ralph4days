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
  status TEXT NOT NULL DEFAULT 'pending' CHECK(status IN ('draft','pending','in_progress','done','blocked','skipped')),
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

-- Task Comments (flat, no JSON)
CREATE TABLE task_comments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  discipline_id INTEGER REFERENCES disciplines(id) ON DELETE SET NULL,
  session_id TEXT,

  verb TEXT NOT NULL CHECK(verb IN ('comment','done','partial','stuck','ask','flag','learned','suggest','blocked')),

  -- Text fields (at least ONE must be populated)
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
  feature_id INTEGER REFERENCES features(id) ON DELETE SET NULL, -- for 'suggest' verb
  detail TEXT,

  -- Answer
  answer TEXT,

  created TEXT NOT NULL DEFAULT (datetime('now')),

  -- Ensure at least one text field is populated
  CHECK(
    text IS NOT NULL OR
    summary IS NOT NULL OR
    remaining IS NOT NULL OR
    reason IS NOT NULL OR
    question IS NOT NULL OR
    what IS NOT NULL OR
    "on" IS NOT NULL
  )
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

-- Recipe configs (normalized)
CREATE TABLE recipe_configs (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  base_recipe TEXT NOT NULL,
  created TEXT,
  updated TEXT
) STRICT;

CREATE TABLE recipe_sections (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  recipe_id INTEGER NOT NULL REFERENCES recipe_configs(id) ON DELETE CASCADE,
  section_name TEXT NOT NULL,
  section_order INTEGER NOT NULL,
  content TEXT NOT NULL,
  UNIQUE(recipe_id, section_name)
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

CREATE INDEX idx_comments_task ON task_comments(task_id);
CREATE INDEX idx_comments_discipline ON task_comments(discipline_id);
CREATE INDEX idx_comments_session ON task_comments(session_id);
CREATE INDEX idx_comments_verb ON task_comments(verb);
CREATE INDEX idx_comments_task_verb ON task_comments(task_id, verb);
CREATE INDEX idx_comments_feature ON task_comments(feature_id);

CREATE INDEX idx_feature_comments_feature ON feature_comments(feature_id);
CREATE INDEX idx_feature_comments_category ON feature_comments(category);
CREATE INDEX idx_feature_comments_discipline ON feature_comments(discipline_id);

CREATE INDEX idx_recipe_sections_recipe ON recipe_sections(recipe_id);

-- Auto-update timestamp trigger
CREATE TRIGGER update_task_timestamp
AFTER UPDATE ON tasks
FOR EACH ROW
WHEN NEW.updated IS NULL OR OLD.updated = NEW.updated
BEGIN
  UPDATE tasks SET updated = datetime('now') WHERE id = NEW.id;
END;
