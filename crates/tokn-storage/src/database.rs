use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OpenFlags, params};
use tokn_domain::{RunId, SourceId};

const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub struct RunRecord {
    pub run_id: String,
    pub source_path: String,
    pub accounting_mode: String,
    pub duplicates_suppressed: i64,
    pub usage_conflicts: i64,
    pub records_seen: i64,
    pub records_valid: i64,
    pub malformed_records: i64,
    pub oversized_records: i64,
    pub truncated_tail: i64,
    pub usage_records: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub cache_write_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub logical_tokens: i64,
    pub invariant_conflicts: i64,
}

pub struct Database {
    conn: Connection,
    path: PathBuf,
}
impl Database {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let conn = Connection::open(path)?;
        configure_connection(&conn)?;
        conn.execute_batch(include_str!("../migrations/0001_initial.sql"))?;
        ensure_usage_conflicts_column(&conn)?;
        ensure_cache_write_input_tokens_column(&conn)?;
        apply_measurement_store_migrations(&conn)?;
        apply_tool_activity_migrations(&conn)?;
        apply_source_version_migrations(&conn)?;
        apply_workspace_git_provenance_migrations(&conn)?;
        apply_task_input_identity_migrations(&conn)?;
        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    pub fn open_read_only(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        configure_connection(&conn)?;
        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn save_run(
        &self,
        run_id: &RunId,
        source_id: &SourceId,
        _source_path: &Path,
        source_kind: &str,
        snapshot_bytes: i64,
        record: &RunRecord,
    ) -> rusqlite::Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let tx = self.conn.unchecked_transaction()?;
        let private_source_ref = format!("private:{}", source_id.0);
        tx.execute(
            "INSERT OR REPLACE INTO runs (
                run_id, created_at_unix, source_path, source_kind,
                accounting_mode, duplicates_suppressed, usage_conflicts,
                records_seen, records_valid, malformed_records,
                oversized_records, truncated_tail, usage_records,
                input_tokens, cached_input_tokens, cache_write_input_tokens,
                output_tokens, reasoning_output_tokens, logical_tokens, invariant_conflicts
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20)",
            params![
                run_id.0,
                now,
                private_source_ref,
                source_kind,
                record.accounting_mode,
                record.duplicates_suppressed,
                record.usage_conflicts,
                record.records_seen,
                record.records_valid,
                record.malformed_records,
                record.oversized_records,
                record.truncated_tail,
                record.usage_records,
                record.input_tokens,
                record.cached_input_tokens,
                record.cache_write_input_tokens,
                record.output_tokens,
                record.reasoning_output_tokens,
                record.logical_tokens,
                record.invariant_conflicts
            ],
        )?;

        tx.execute(
            "INSERT OR REPLACE INTO sources (
                source_id, run_id, canonical_path, snapshot_bytes, source_kind
             ) VALUES (?1,?2,?3,?4,?5)",
            params![
                source_id.0,
                run_id.0,
                private_source_ref,
                snapshot_bytes,
                source_kind
            ],
        )?;

        tx.commit()
    }

    pub fn run_count(&self) -> rusqlite::Result<i64> {
        self.conn
            .query_row("SELECT COUNT(*) FROM runs", [], |row| row.get(0))
    }

    pub fn latest_run(&self) -> rusqlite::Result<Option<RunRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT run_id, source_path, accounting_mode, duplicates_suppressed, usage_conflicts,
                    records_seen, records_valid, malformed_records,
                    oversized_records, truncated_tail, usage_records,
                    input_tokens, cached_input_tokens, cache_write_input_tokens,
                    output_tokens, reasoning_output_tokens, logical_tokens, invariant_conflicts
             FROM runs ORDER BY created_at_unix DESC LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };

        Ok(Some(RunRecord {
            run_id: row.get(0)?,
            source_path: row.get(1)?,
            accounting_mode: row.get(2)?,
            duplicates_suppressed: row.get(3)?,
            usage_conflicts: row.get(4)?,
            records_seen: row.get(5)?,
            records_valid: row.get(6)?,
            malformed_records: row.get(7)?,
            oversized_records: row.get(8)?,
            truncated_tail: row.get(9)?,
            usage_records: row.get(10)?,
            input_tokens: row.get(11)?,
            cached_input_tokens: row.get(12)?,
            cache_write_input_tokens: row.get(13)?,
            output_tokens: row.get(14)?,
            reasoning_output_tokens: row.get(15)?,
            logical_tokens: row.get(16)?,
            invariant_conflicts: row.get(17)?,
        }))
    }
}

fn configure_connection(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;
    conn.busy_timeout(SQLITE_BUSY_TIMEOUT)?;
    Ok(())
}

fn ensure_usage_conflicts_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(runs)")?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if !names.iter().any(|name| name == "usage_conflicts") {
        conn.execute(
            "ALTER TABLE runs ADD COLUMN usage_conflicts INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

fn apply_measurement_store_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let current = {
        let mut stmt = conn.prepare(
            "SELECT value FROM schema_meta WHERE key='measurement_store_schema_version'",
        )?;
        let mut rows = stmt.query([])?;
        rows.next()?
            .map(|row| row.get::<_, String>(0))
            .transpose()?
    };

    match current.as_deref() {
        Some("2") => Ok(()),
        None | Some("1") => {
            conn.execute_batch(include_str!("../migrations/0002_measurement_store.sql"))?;
            conn.execute_batch("VACUUM;")?;
            Ok(())
        }
        Some(_) => Err(rusqlite::Error::InvalidQuery),
    }
}

fn apply_tool_activity_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let current = {
        let mut stmt =
            conn.prepare("SELECT value FROM schema_meta WHERE key='tool_activity_schema_version'")?;
        let mut rows = stmt.query([])?;
        rows.next()?
            .map(|row| row.get::<_, String>(0))
            .transpose()?
    };

    match current.as_deref() {
        Some("3") => Ok(()),
        Some("2") => {
            conn.execute_batch(include_str!("../migrations/0005_tool_activity_timing.sql"))
        }
        Some("1") => {
            conn.execute_batch(include_str!("../migrations/0004_context_identity.sql"))?;
            conn.execute_batch(include_str!("../migrations/0005_tool_activity_timing.sql"))
        }
        None => {
            conn.execute_batch(include_str!("../migrations/0003_tool_activity.sql"))?;
            conn.execute_batch(include_str!("../migrations/0004_context_identity.sql"))?;
            conn.execute_batch(include_str!("../migrations/0005_tool_activity_timing.sql"))
        }
        Some(_) => Err(rusqlite::Error::InvalidQuery),
    }
}

fn apply_source_version_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let current = {
        let mut stmt = conn
            .prepare("SELECT value FROM schema_meta WHERE key='source_version_schema_version'")?;
        let mut rows = stmt.query([])?;
        rows.next()?
            .map(|row| row.get::<_, String>(0))
            .transpose()?
    };

    match current.as_deref() {
        Some("1") => Ok(()),
        None => conn.execute_batch(include_str!("../migrations/0006_source_versions.sql")),
        Some(_) => Err(rusqlite::Error::InvalidQuery),
    }
}

fn apply_workspace_git_provenance_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let current = {
        let mut stmt = conn.prepare(
            "SELECT value FROM schema_meta WHERE key='workspace_git_provenance_schema_version'",
        )?;
        let mut rows = stmt.query([])?;
        rows.next()?
            .map(|row| row.get::<_, String>(0))
            .transpose()?
    };

    match current.as_deref() {
        Some("1") => Ok(()),
        None => conn.execute_batch(include_str!(
            "../migrations/0007_workspace_git_provenance.sql"
        )),
        Some(_) => Err(rusqlite::Error::InvalidQuery),
    }
}

fn apply_task_input_identity_migrations(conn: &Connection) -> rusqlite::Result<()> {
    let current = {
        let mut stmt = conn.prepare(
            "SELECT value FROM schema_meta WHERE key='task_input_identity_schema_version'",
        )?;
        let mut rows = stmt.query([])?;
        rows.next()?
            .map(|row| row.get::<_, String>(0))
            .transpose()?
    };

    match current.as_deref() {
        Some("1") => Ok(()),
        None => conn.execute_batch(include_str!("../migrations/0008_task_input_identity.sql")),
        Some(_) => Err(rusqlite::Error::InvalidQuery),
    }
}

fn ensure_cache_write_input_tokens_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("PRAGMA table_info(runs)")?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if !names.iter().any(|name| name == "cache_write_input_tokens") {
        conn.execute(
            "ALTER TABLE runs ADD COLUMN cache_write_input_tokens INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn sample(run_id: &str) -> RunRecord {
        RunRecord {
            run_id: run_id.to_string(),
            source_path: "fixture.jsonl".into(),
            accounting_mode: "fixture".into(),
            duplicates_suppressed: 0,
            usage_conflicts: 0,
            records_seen: 1,
            records_valid: 1,
            malformed_records: 0,
            oversized_records: 0,
            truncated_tail: 0,
            usage_records: 1,
            input_tokens: 10,
            cached_input_tokens: 5,
            cache_write_input_tokens: 0,
            output_tokens: 2,
            reasoning_output_tokens: 1,
            logical_tokens: 12,
            invariant_conflicts: 0,
        }
    }

    #[test]
    fn database_open_enables_foreign_keys_and_busy_timeout() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-connection-guards-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let db = Database::open(&path).unwrap();

        let foreign_keys: i64 = db
            .connection()
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        let busy_timeout_ms: i64 = db
            .connection()
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .unwrap();

        assert_eq!(foreign_keys, 1);
        assert_eq!(busy_timeout_ms, 5_000);
        drop(db);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn database_open_read_only_does_not_create_missing_store() {
        let parent = std::env::temp_dir().join(format!(
            "tokn-storage-read-only-missing-{}",
            std::process::id()
        ));
        let path = parent.join("missing.sqlite3");
        let _ = fs::remove_dir_all(&parent);

        assert!(Database::open_read_only(&path).is_err());
        assert!(!path.exists());
        assert!(!parent.exists());
    }

    #[test]
    fn database_open_read_only_does_not_migrate_uninitialized_store() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-read-only-uninitialized-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let seed = Connection::open(&path).unwrap();
        seed.execute_batch("CREATE TABLE sentinel(value INTEGER NOT NULL);")
            .unwrap();
        drop(seed);
        let before = fs::read(&path).unwrap();

        let read_only = Database::open_read_only(&path).unwrap();
        assert!(read_only.run_count().is_err());
        drop(read_only);

        let after = fs::read(&path).unwrap();
        assert_eq!(before, after);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn database_open_read_only_rejects_writes_without_mutating_store() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-read-only-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let writable = Database::open(&path).unwrap();
        assert_eq!(writable.run_count().unwrap(), 0);
        drop(writable);
        let before = fs::read(&path).unwrap();

        let read_only = Database::open_read_only(&path).unwrap();
        assert_eq!(read_only.run_count().unwrap(), 0);
        assert!(
            read_only
                .connection()
                .execute("DELETE FROM runs", [])
                .is_err()
        );
        drop(read_only);

        let after = fs::read(&path).unwrap();
        assert_eq!(before, after);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn database_open_enforces_source_run_foreign_key() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-foreign-key-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let db = Database::open(&path).unwrap();

        let error = db
            .connection()
            .execute(
                "INSERT INTO sources(source_id, run_id, canonical_path, snapshot_bytes, source_kind)
                 VALUES ('orphan-source','missing-run','private:orphan-source',1,'fixture')",
                [],
            )
            .expect_err("foreign key must reject an orphan source row");

        assert!(matches!(error, rusqlite::Error::SqliteFailure(_, _)));
        drop(db);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn same_run_id_is_idempotent_in_database() {
        let path =
            std::env::temp_dir().join(format!("tokn-storage-{}.sqlite3", std::process::id()));
        let _ = fs::remove_file(&path);
        let db = Database::open(&path).unwrap();
        let run = RunId("run-fixed".into());
        let source = SourceId("src-fixed".into());
        let record = sample(&run.0);

        db.save_run(
            &run,
            &source,
            Path::new("fixture.jsonl"),
            "fixture",
            42,
            &record,
        )
        .unwrap();
        db.save_run(
            &run,
            &source,
            Path::new("fixture.jsonl"),
            "fixture",
            42,
            &record,
        )
        .unwrap();

        assert_eq!(db.run_count().unwrap(), 1);
        let latest = db.latest_run().unwrap().expect("latest run");
        assert_eq!(latest.source_path, "private:src-fixed");
        drop(db);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v2_migration_physically_redacts_legacy_paths() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-legacy-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);
        let private_path = format!(
            "C:\\{}\\legacy-private\\Secret Project\\session.jsonl",
            "Users"
        );

        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
                .unwrap();
            conn.execute(
                "INSERT INTO runs(
                    run_id, created_at_unix, source_path, source_kind, accounting_mode,
                    duplicates_suppressed, usage_conflicts, records_seen, records_valid,
                    malformed_records, oversized_records, truncated_tail, usage_records,
                    input_tokens, cached_input_tokens, cache_write_input_tokens, output_tokens,
                    reasoning_output_tokens, logical_tokens, invariant_conflicts
                 ) VALUES (
                    'legacy-run',0,?1,'codex-session','legacy',0,0,1,1,0,0,0,1,10,5,0,2,1,12,0
                 )",
                params![&private_path],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO sources(source_id, run_id, canonical_path, snapshot_bytes, source_kind)
                 VALUES ('legacy-source','legacy-run',?1,42,'codex-session')",
                params![&private_path],
            )
            .unwrap();
        }

        let before = fs::read(&path).unwrap();
        assert!(String::from_utf8_lossy(&before).contains("legacy-private"));

        let db = Database::open(&path).unwrap();
        let latest = db.latest_run().unwrap().expect("legacy run");
        assert_eq!(latest.source_path, "private:legacy-run");
        drop(db);

        let after = fs::read(&path).unwrap();
        let text = String::from_utf8_lossy(&after);
        assert!(!text.contains(&private_path));
        assert!(!text.contains("legacy-private"));
        assert!(!text.contains("Secret Project"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn refuses_unknown_future_measurement_store_version() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-future-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
                .unwrap();
            conn.execute(
                "INSERT INTO schema_meta(key, value)
                 VALUES ('measurement_store_schema_version', '99')",
                [],
            )
            .unwrap();
        }

        assert!(matches!(
            Database::open(&path),
            Err(rusqlite::Error::InvalidQuery)
        ));
        let _ = fs::remove_file(path);
    }
    #[test]
    fn migrates_tool_activity_v1_to_timing_v3() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-activity-v1-v3-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
                .unwrap();
            conn.execute_batch(include_str!("../migrations/0002_measurement_store.sql"))
                .unwrap();
            conn.execute_batch(include_str!("../migrations/0003_tool_activity.sql"))
                .unwrap();
        }

        let db = Database::open(&path).unwrap();
        assert_eq!(
            db.tool_activity_schema_version().unwrap().as_deref(),
            Some("3")
        );
        drop(db);

        let conn = Connection::open(&path).unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(tool_activity_v1)").unwrap();
        let names = stmt
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert!(names.iter().any(|name| name == "source_stable_id"));
        assert!(names.iter().any(|name| name == "source_identity_coverage"));
        assert!(names.iter().any(|name| name == "content_fingerprint"));
        assert!(names.iter().any(|name| name == "content_identity_coverage"));
        assert!(names.iter().any(|name| name == "observed_at"));
        drop(stmt);
        drop(conn);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn refuses_unknown_future_tool_activity_version() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-future-activity-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(include_str!("../migrations/0001_initial.sql"))
                .unwrap();
            conn.execute_batch(include_str!("../migrations/0002_measurement_store.sql"))
                .unwrap();
            conn.execute(
                "INSERT INTO schema_meta(key, value)
                 VALUES ('tool_activity_schema_version', '99')",
                [],
            )
            .unwrap();
        }

        assert!(matches!(
            Database::open(&path),
            Err(rusqlite::Error::InvalidQuery)
        ));
        let _ = fs::remove_file(path);
    }
    #[test]
    fn creates_task_input_identity_schema_v1_and_refuses_future_version() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-task-input-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        let db = Database::open(&path).expect("open db");
        let version: String = db
            .connection()
            .query_row(
                "SELECT value FROM schema_meta WHERE key='task_input_identity_schema_version'",
                [],
                |row| row.get(0),
            )
            .expect("schema version");
        assert_eq!(version, "1");
        drop(db);

        let conn = Connection::open(&path).expect("reopen raw db");
        conn.execute(
            "UPDATE schema_meta SET value='99' WHERE key='task_input_identity_schema_version'",
            [],
        )
        .expect("write future version");
        drop(conn);

        assert!(matches!(
            Database::open(&path),
            Err(rusqlite::Error::InvalidQuery)
        ));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn creates_source_version_schema_v1_and_refuses_future_version() {
        let path = std::env::temp_dir().join(format!(
            "tokn-storage-source-version-{}.sqlite3",
            std::process::id()
        ));
        let _ = fs::remove_file(&path);

        let db = Database::open(&path).expect("open db");
        let version: String = db
            .connection()
            .query_row(
                "SELECT value FROM schema_meta WHERE key='source_version_schema_version'",
                [],
                |row| row.get(0),
            )
            .expect("schema version");
        assert_eq!(version, "1");
        drop(db);

        let conn = Connection::open(&path).expect("reopen raw db");
        conn.execute(
            "UPDATE schema_meta SET value='99' WHERE key='source_version_schema_version'",
            [],
        )
        .expect("write future version");
        drop(conn);

        assert!(matches!(
            Database::open(&path),
            Err(rusqlite::Error::InvalidQuery)
        ));
        let _ = fs::remove_file(path);
    }
}
