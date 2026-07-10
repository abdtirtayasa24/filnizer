CREATE TABLE IF NOT EXISTS organizer_rules (
  id TEXT PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL,
  value TEXT NOT NULL,
  category TEXT NOT NULL,
  created_at_unix_ms INTEGER NOT NULL
);
