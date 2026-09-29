PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS schema_meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runs (
  run_id TEXT PRIMARY KEY,
  created_at_unix INTEGER NOT NULL,
  source_path TEXT NOT NULL,
  source_kind TEXT NOT NULL,
  accounting_mode TEXT NOT NULL,
  duplicates_suppressed INTEGER NOT NULL DEFAULT 0,
  usage_conflicts INTEGER NOT NULL DEFAULT 0,
  records_seen INTEGER NOT NULL,
  records_valid INTEGER NOT NULL,
  malformed_records INTEGER NOT NULL,
  oversized_records INTEGER NOT NULL,
  truncated_tail INTEGER NOT NULL,
  usage_records INTEGER NOT NULL,
  input_tokens INTEGER NOT NULL,
  cached_input_tokens INTEGER NOT NULL,
  cache_write_input_tokens INTEGER NOT NULL DEFAULT 0,
  output_tokens INTEGER NOT NULL,
  reasoning_output_tokens INTEGER NOT NULL,
  logical_tokens INTEGER NOT NULL,
  invariant_conflicts INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sources (
  source_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  canonical_path TEXT NOT NULL,
  snapshot_bytes INTEGER NOT NULL,
  source_kind TEXT NOT NULL,
  FOREIGN KEY(run_id) REFERENCES runs(run_id)
);

CREATE INDEX IF NOT EXISTS idx_runs_created
ON runs(created_at_unix DESC);
