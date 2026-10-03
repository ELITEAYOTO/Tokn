
PRAGMA foreign_keys = ON;

ALTER TABLE tool_activity_v1 ADD COLUMN source_stable_id TEXT;
ALTER TABLE tool_activity_v1 ADD COLUMN source_identity_coverage TEXT NOT NULL DEFAULT 'NOT_CAPTURED';
ALTER TABLE tool_activity_v1 ADD COLUMN content_fingerprint TEXT;
ALTER TABLE tool_activity_v1 ADD COLUMN content_identity_coverage TEXT NOT NULL DEFAULT 'NOT_CAPTURED';

CREATE INDEX IF NOT EXISTS idx_tool_activity_v1_content
ON tool_activity_v1(content_fingerprint, category)
WHERE content_fingerprint IS NOT NULL;

INSERT INTO schema_meta(key, value)
VALUES ('tool_activity_schema_version', '2')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
