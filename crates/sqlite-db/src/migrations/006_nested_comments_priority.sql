-- Migration 006: Add nested comments (2-layer threading) and comment priority
-- Supports human replies to any comment (regular or signal), with priority field

ALTER TABLE task_comments ADD COLUMN parent_comment_id INTEGER REFERENCES task_comments(id) ON DELETE CASCADE;
ALTER TABLE task_comments ADD COLUMN priority TEXT CHECK(priority IN ('critical', 'high', 'medium', 'low', 'none') OR priority IS NULL);

-- Index for fetching replies efficiently
CREATE INDEX idx_task_comments_parent ON task_comments(parent_comment_id) WHERE parent_comment_id IS NOT NULL;
