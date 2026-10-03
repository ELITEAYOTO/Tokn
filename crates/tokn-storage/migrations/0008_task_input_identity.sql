CREATE TABLE IF NOT EXISTS task_input_identity_v1 (
    task_input_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL UNIQUE,
    coverage TEXT NOT NULL CHECK(coverage IN ('OBSERVED', 'PARTIAL', 'NOT_CAPTURED', 'UNKNOWN')),
    task_fingerprint TEXT,
    bytes INTEGER,
    created_at_unix INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES measurement_runs_v2(run_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_task_input_identity_v1_run
ON task_input_identity_v1(run_id);

INSERT INTO schema_meta(key, value)
VALUES ('task_input_identity_schema_version', '1')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
