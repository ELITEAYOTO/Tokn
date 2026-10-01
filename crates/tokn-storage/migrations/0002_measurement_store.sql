PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS projects_v2 (
  project_id TEXT PRIMARY KEY,
  identity_version INTEGER NOT NULL,
  created_at_unix INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS workspaces_v2 (
  workspace_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  parent_workspace_id TEXT,
  snapshot_fingerprint TEXT,
  identity_version INTEGER NOT NULL,
  created_at_unix INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects_v2(project_id),
  FOREIGN KEY(parent_workspace_id) REFERENCES workspaces_v2(workspace_id)
);

CREATE TABLE IF NOT EXISTS runtime_profiles_v2 (
  profile_id TEXT PRIMARY KEY,
  schema_version INTEGER NOT NULL,
  observed_at TEXT NOT NULL,
  runtime_kind TEXT NOT NULL,
  runtime_version TEXT,
  app_version TEXT,
  model TEXT,
  model_provider TEXT,
  configuration_complete TEXT NOT NULL,
  profile_json TEXT NOT NULL,
  created_at_unix INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS measurement_runs_v2 (
  run_id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL,
  workspace_id TEXT NOT NULL,
  profile_id TEXT,
  measurement_contract_version INTEGER NOT NULL,
  evidence_layout_version INTEGER NOT NULL,
  root_thread_id TEXT NOT NULL,
  root_terminal TEXT NOT NULL,
  source_health TEXT NOT NULL,
  validity_verdict TEXT,
  quality_status TEXT,
  agent_count INTEGER NOT NULL,
  usage_records INTEGER NOT NULL,
  input_tokens INTEGER NOT NULL,
  cached_input_tokens INTEGER NOT NULL,
  cache_write_input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  reasoning_output_tokens INTEGER NOT NULL,
  input_known INTEGER NOT NULL,
  cached_input_known INTEGER NOT NULL,
  cache_write_input_known INTEGER NOT NULL,
  output_known INTEGER NOT NULL,
  reasoning_output_known INTEGER NOT NULL,
  logical_tokens INTEGER,
  created_at_unix INTEGER NOT NULL,
  FOREIGN KEY(project_id) REFERENCES projects_v2(project_id),
  FOREIGN KEY(workspace_id) REFERENCES workspaces_v2(workspace_id),
  FOREIGN KEY(profile_id) REFERENCES runtime_profiles_v2(profile_id)
);

CREATE TABLE IF NOT EXISTS agents_v2 (
  run_id TEXT NOT NULL,
  thread_id TEXT NOT NULL,
  parent_thread_id TEXT,
  depth INTEGER NOT NULL,
  terminal_status TEXT NOT NULL,
  duration_ms INTEGER,
  usage_records INTEGER NOT NULL,
  input_tokens INTEGER NOT NULL,
  cached_input_tokens INTEGER NOT NULL,
  cache_write_input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  reasoning_output_tokens INTEGER NOT NULL,
  input_known INTEGER NOT NULL,
  cached_input_known INTEGER NOT NULL,
  cache_write_input_known INTEGER NOT NULL,
  output_known INTEGER NOT NULL,
  reasoning_output_known INTEGER NOT NULL,
  logical_tokens INTEGER,
  PRIMARY KEY(run_id, thread_id),
  FOREIGN KEY(run_id) REFERENCES measurement_runs_v2(run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS provenance_v2 (
  source_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  source_fingerprint TEXT NOT NULL,
  snapshot_bytes INTEGER,
  adapter_name TEXT,
  adapter_version TEXT,
  created_at_unix INTEGER NOT NULL,
  UNIQUE(run_id, source_fingerprint),
  FOREIGN KEY(run_id) REFERENCES measurement_runs_v2(run_id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS rate_limit_snapshots_v2 (
  snapshot_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  observed_at TEXT NOT NULL,
  limit_id TEXT,
  primary_used_percent TEXT,
  primary_window_minutes INTEGER,
  primary_resets_at TEXT,
  secondary_used_percent TEXT,
  secondary_window_minutes INTEGER,
  secondary_resets_at TEXT,
  rate_limit_reached_type TEXT,
  created_at_unix INTEGER NOT NULL,
  FOREIGN KEY(run_id) REFERENCES measurement_runs_v2(run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_measurement_runs_v2_created
ON measurement_runs_v2(created_at_unix DESC);

CREATE INDEX IF NOT EXISTS idx_measurement_runs_v2_project
ON measurement_runs_v2(project_id, created_at_unix DESC);

CREATE INDEX IF NOT EXISTS idx_agents_v2_run
ON agents_v2(run_id, depth, thread_id);

UPDATE runs
SET source_path = 'private:' || run_id
WHERE source_path NOT LIKE 'private:%';

UPDATE sources
SET canonical_path = 'private:' || source_id
WHERE canonical_path NOT LIKE 'private:%';

INSERT INTO schema_meta(key, value)
VALUES ('measurement_store_schema_version', '2')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
