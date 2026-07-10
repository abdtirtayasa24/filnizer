CREATE TABLE IF NOT EXISTS settings (
  key TEXT PRIMARY KEY NOT NULL,
  value TEXT NOT NULL,
  updated_at_unix_ms INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS jobs (
  id TEXT PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL,
  status TEXT NOT NULL,
  name TEXT NOT NULL,
  total_files INTEGER NOT NULL DEFAULT 0,
  completed_files INTEGER NOT NULL DEFAULT 0,
  created_at_unix_ms INTEGER NOT NULL,
  updated_at_unix_ms INTEGER NOT NULL,
  error_message TEXT
);

CREATE TABLE IF NOT EXISTS operation_plans (
  id TEXT PRIMARY KEY NOT NULL,
  job_id TEXT,
  status TEXT NOT NULL,
  plan_json TEXT NOT NULL,
  created_at_unix_ms INTEGER NOT NULL,
  FOREIGN KEY (job_id) REFERENCES jobs(id)
);

CREATE TABLE IF NOT EXISTS file_results (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  job_id TEXT NOT NULL,
  source_path TEXT NOT NULL,
  target_path TEXT,
  status TEXT NOT NULL,
  message TEXT,
  updated_at_unix_ms INTEGER NOT NULL,
  FOREIGN KEY (job_id) REFERENCES jobs(id)
);
