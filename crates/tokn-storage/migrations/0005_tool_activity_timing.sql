PRAGMA foreign_keys = ON;

ALTER TABLE tool_activity_v1 ADD COLUMN observed_at TEXT;

INSERT INTO schema_meta(key, value)
VALUES ('tool_activity_schema_version', '3')
ON CONFLICT(key) DO UPDATE SET value = excluded.value;
