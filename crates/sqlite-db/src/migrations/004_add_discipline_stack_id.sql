-- Add stack_id to disciplines table for team roster grouping
ALTER TABLE disciplines ADD COLUMN stack_id INTEGER DEFAULT NULL;
