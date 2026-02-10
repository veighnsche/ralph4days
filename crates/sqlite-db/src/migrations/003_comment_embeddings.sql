CREATE TABLE comment_embeddings (
  comment_id INTEGER PRIMARY KEY REFERENCES feature_comments(id) ON DELETE CASCADE,
  embedding BLOB NOT NULL,
  embedding_model TEXT NOT NULL,
  embedding_hash TEXT NOT NULL
) STRICT;
