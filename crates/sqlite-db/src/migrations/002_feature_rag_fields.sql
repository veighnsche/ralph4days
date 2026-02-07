-- Add RAG-related fields to features table
ALTER TABLE features ADD COLUMN architecture TEXT;
ALTER TABLE features ADD COLUMN boundaries TEXT;
ALTER TABLE features ADD COLUMN learnings TEXT DEFAULT '[]';
ALTER TABLE features ADD COLUMN dependencies TEXT DEFAULT '[]';
