-- Phase A: Feature Comments as the knowledge layer.
-- Migrates architecture/boundaries/learnings from features into feature_comments table,
-- adds status column, and drops the old columns.

CREATE TABLE feature_comments (
  id INTEGER PRIMARY KEY,
  feature_name TEXT NOT NULL REFERENCES features(name) ON DELETE CASCADE,
  category TEXT NOT NULL,
  author TEXT NOT NULL,
  discipline TEXT REFERENCES disciplines(name) ON DELETE SET NULL,
  agent_task_id INTEGER,
  body TEXT NOT NULL,
  reason TEXT,
  source_iteration INTEGER,
  created TEXT,
  updated TEXT
) STRICT;

CREATE INDEX idx_feature_comments_feature ON feature_comments(feature_name);

-- Migrate architecture blobs into feature_comments
INSERT INTO feature_comments (feature_name, category, author, body, reason, created)
SELECT name, 'architecture', 'system', architecture, 'Migrated from features.architecture column', created
FROM features WHERE architecture IS NOT NULL AND architecture != '';

-- Migrate boundaries blobs into feature_comments
INSERT INTO feature_comments (feature_name, category, author, body, reason, created)
SELECT name, 'boundary', 'system', boundaries, 'Migrated from features.boundaries column', created
FROM features WHERE boundaries IS NOT NULL AND boundaries != '';

-- Migrate learnings JSON array into feature_comments
INSERT INTO feature_comments (feature_name, category, author, body, reason, created)
SELECT
  f.name,
  'learning',
  CASE json_extract(j.value, '$.source')
    WHEN 'human' THEN 'human'
    WHEN 'agent' THEN 'agent'
    WHEN 'opus_reviewed' THEN 'agent'
    ELSE 'system'
  END,
  json_extract(j.value, '$.text'),
  json_extract(j.value, '$.reason'),
  json_extract(j.value, '$.created')
FROM features f, json_each(f.learnings) j
WHERE f.learnings != '[]';

-- Recreate features table without architecture/boundaries/learnings, adding status
CREATE TABLE features_new (
  name TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  acronym TEXT NOT NULL UNIQUE,
  description TEXT,
  created TEXT,
  knowledge_paths TEXT DEFAULT '[]',
  context_files TEXT DEFAULT '[]',
  dependencies TEXT DEFAULT '[]',
  status TEXT NOT NULL DEFAULT 'active'
) STRICT;

INSERT INTO features_new (name, display_name, acronym, description, created, knowledge_paths, context_files, dependencies)
SELECT name, display_name, acronym, description, created, knowledge_paths, context_files, dependencies
FROM features;

DROP TABLE features;
ALTER TABLE features_new RENAME TO features;
