CREATE TABLE recipe_configs (
  name TEXT PRIMARY KEY,
  base_recipe TEXT NOT NULL,
  section_order TEXT NOT NULL DEFAULT '[]',
  sections TEXT NOT NULL DEFAULT '{}',
  created TEXT,
  updated TEXT
) STRICT;
