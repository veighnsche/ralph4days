-- Migration 007: Add foreign key constraint for task_comments.author
--
-- Problem: author field is free-text, causing data quality issues and requiring
-- runtime lookups to get capitalized discipline names.
--
-- Solution: Make author a foreign key to disciplines(name) and JOIN when querying.

-- SQLite doesn't support ALTER TABLE to add FK constraints, so we need to recreate the table
CREATE TABLE task_comments_new (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  task_id INTEGER NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  session_id TEXT,
  author TEXT REFERENCES disciplines(name) ON DELETE RESTRICT,
  signal_verb TEXT CHECK(signal_verb IN ('done','partial','stuck','ask','flag','learned','suggest','blocked') OR signal_verb IS NULL),
  signal_payload TEXT,
  signal_answered TEXT,
  parent_comment_id INTEGER REFERENCES task_comments_new(id) ON DELETE CASCADE,
  priority TEXT,
  body TEXT NOT NULL,
  created TEXT NOT NULL DEFAULT (datetime('now'))
) STRICT;

-- Copy data, normalizing author names to match discipline names
INSERT INTO task_comments_new (id, task_id, session_id, author, signal_verb, signal_payload, signal_answered, parent_comment_id, priority, body, created)
SELECT
  id,
  task_id,
  session_id,
  CASE
    WHEN lower(author) IN (SELECT lower(name) FROM disciplines)
      THEN (SELECT name FROM disciplines WHERE lower(disciplines.name) = lower(task_comments.author))
    ELSE NULL
  END as author,
  signal_verb,
  signal_payload,
  signal_answered,
  parent_comment_id,
  priority,
  body,
  created
FROM task_comments;

-- Replace old table
DROP TABLE task_comments;
ALTER TABLE task_comments_new RENAME TO task_comments;
