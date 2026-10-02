PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS tool_activity_v1 (
  activity_id TEXT PRIMARY KEY,
  run_id TEXT NOT NULL,
  thread_id TEXT NOT NULL,
  agent_ordinal INTEGER NOT NULL,
  kind TEXT NOT NULL,
  tool_name TEXT,
  category TEXT NOT NULL,
  surface TEXT NOT NULL,
  requester_type TEXT,
  status TEXT NOT NULL,
  started_seq INTEGER,
  ended_seq INTEGER,
  invocation_payload_bytes INTEGER,
  result_payload_bytes INTEGER,
  result_output_chars INTEGER,
  max_output_tokens INTEGER,
  original_token_count INTEGER,
  operation_fingerprint TEXT,
  workdir_fingerprint TEXT,
  parse_error_present INTEGER NOT NULL,
  created_at_unix INTEGER NOT NULL,
  UNIQUE(run_id, thread_id, agent_ordinal),
  FOREIGN KEY(run_id, thread_id)
    REFERENCES agents_v2(run_id, thread_id)
    ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tool_activity_v1_run
ON tool_activity_v1(run_id, thread_id, agent_ordinal);

CREATE INDEX IF NOT EXISTS idx_tool_activity_v1_category
ON tool_activity_v1(category, run_id);

CREATE INDEX IF NOT EXISTS idx_tool_activity_v1_operation
ON tool_activity_v1(operation_fingerprint, category)
WHERE operation_fingerprint IS NOT NULL;

INSERT INTO schema_meta(key, value)
VALUES ('tool_activity_schema_version', '1')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
