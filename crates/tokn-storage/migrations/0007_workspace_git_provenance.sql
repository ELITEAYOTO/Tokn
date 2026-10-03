PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS workspace_git_provenance_v1 (
    provenance_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    boundary TEXT NOT NULL CHECK(boundary IN ('BEFORE', 'AFTER')),
    coverage TEXT NOT NULL CHECK(coverage IN ('OBSERVED', 'NOT_CAPTURED', 'UNKNOWN')),
    head_fingerprint TEXT,
    dirty INTEGER CHECK(dirty IN (0, 1)),
    snapshot_observed_at TEXT,
    created_at_unix INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES measurement_runs_v2(run_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_workspace_git_provenance_v1_run_boundary
ON workspace_git_provenance_v1(run_id, boundary);

INSERT INTO schema_meta(key, value)
VALUES ('workspace_git_provenance_schema_version', '1')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
