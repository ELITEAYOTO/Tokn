PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS source_versions_v1 (
    version_id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL,
    source_stable_id TEXT NOT NULL,
    boundary TEXT NOT NULL CHECK(boundary IN ('BEFORE', 'AFTER')),
    version_fingerprint TEXT NOT NULL,
    snapshot_observed_at TEXT,
    bytes INTEGER NOT NULL,
    created_at_unix INTEGER NOT NULL,
    FOREIGN KEY(run_id) REFERENCES measurement_runs_v2(run_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_source_versions_v1_run_source
ON source_versions_v1(run_id, source_stable_id, boundary);

INSERT INTO schema_meta(key, value)
VALUES ('source_version_schema_version', '1')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
