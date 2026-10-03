use rusqlite::Connection;
use serde::{Deserialize, Serialize};

pub const SHADOW_SQLITE_CAPABILITY_SCHEMA_VERSION: u64 = 1;
pub const DIRECT_SCAN_FALLBACK_BACKEND_ID: &str = "DIRECT_SCAN_V0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowCapabilityState {
    ObservedAvailable,
    ObservedUnavailable,
    NotApplicable,
    Unknown,
}

impl ShadowCapabilityState {
    pub const fn is_available(self) -> bool {
        matches!(self, Self::ObservedAvailable)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqliteFts5CapabilityReport {
    pub schema_version: u64,
    pub sqlite_version: Option<String>,
    pub compile_option_fts5: ShadowCapabilityState,
    pub fts5: ShadowCapabilityState,
    pub unicode61: ShadowCapabilityState,
    pub trigram: ShadowCapabilityState,
    pub contentless: ShadowCapabilityState,
    pub contentless_delete: ShadowCapabilityState,
    pub integrity_check: ShadowCapabilityState,
    pub extension_loading_attempted: bool,
    pub fallback_backend_id: String,
    pub reason_codes: Vec<String>,
}

pub fn probe_sqlite_fts5_capabilities() -> SqliteFts5CapabilityReport {
    let connection = match Connection::open_in_memory() {
        Ok(connection) => connection,
        Err(_) => {
            return unavailable_runtime_report("SQLITE_OPEN_IN_MEMORY_FAILED");
        }
    };

    let mut reason_codes = Vec::new();
    let sqlite_version = match connection.query_row("SELECT sqlite_version()", [], |row| row.get(0))
    {
        Ok(version) => Some(version),
        Err(_) => {
            reason_codes.push("SQLITE_VERSION_QUERY_FAILED".to_owned());
            None
        }
    };
    let compile_option_fts5 = match connection.query_row(
        "SELECT sqlite_compileoption_used('ENABLE_FTS5')",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(value) if value != 0 => ShadowCapabilityState::ObservedAvailable,
        Ok(_) => ShadowCapabilityState::ObservedUnavailable,
        Err(_) => {
            reason_codes.push("SQLITE_COMPILE_OPTION_QUERY_FAILED".to_owned());
            ShadowCapabilityState::Unknown
        }
    };

    let fts5 = probe_basic_fts5(&connection);
    if let Some(reason) = fts5.reason_code {
        reason_codes.push(reason.to_owned());
    }

    let (unicode61, trigram, contentless, contentless_delete, integrity_check) =
        if fts5.state == ShadowCapabilityState::ObservedAvailable {
            let unicode61 = probe_unicode61(&connection);
            let trigram = probe_trigram(&connection);
            let contentless = probe_contentless(&connection);
            let contentless_delete = probe_contentless_delete(&connection);
            let integrity_check = probe_integrity_check(&connection);
            for outcome in [
                unicode61,
                trigram,
                contentless,
                contentless_delete,
                integrity_check,
            ] {
                if let Some(reason) = outcome.reason_code {
                    reason_codes.push(reason.to_owned());
                }
            }
            (
                unicode61.state,
                trigram.state,
                contentless.state,
                contentless_delete.state,
                integrity_check.state,
            )
        } else {
            let dependent = if fts5.state == ShadowCapabilityState::ObservedUnavailable {
                ShadowCapabilityState::ObservedUnavailable
            } else {
                ShadowCapabilityState::Unknown
            };
            (dependent, dependent, dependent, dependent, dependent)
        };

    SqliteFts5CapabilityReport {
        schema_version: SHADOW_SQLITE_CAPABILITY_SCHEMA_VERSION,
        sqlite_version,
        compile_option_fts5,
        fts5: fts5.state,
        unicode61,
        trigram,
        contentless,
        contentless_delete,
        integrity_check,
        extension_loading_attempted: false,
        fallback_backend_id: DIRECT_SCAN_FALLBACK_BACKEND_ID.to_owned(),
        reason_codes,
    }
}

fn unavailable_runtime_report(reason_code: &str) -> SqliteFts5CapabilityReport {
    SqliteFts5CapabilityReport {
        schema_version: SHADOW_SQLITE_CAPABILITY_SCHEMA_VERSION,
        sqlite_version: None,
        compile_option_fts5: ShadowCapabilityState::Unknown,
        fts5: ShadowCapabilityState::Unknown,
        unicode61: ShadowCapabilityState::Unknown,
        trigram: ShadowCapabilityState::Unknown,
        contentless: ShadowCapabilityState::Unknown,
        contentless_delete: ShadowCapabilityState::Unknown,
        integrity_check: ShadowCapabilityState::Unknown,
        extension_loading_attempted: false,
        fallback_backend_id: DIRECT_SCAN_FALLBACK_BACKEND_ID.to_owned(),
        reason_codes: vec![reason_code.to_owned()],
    }
}

#[derive(Debug, Clone, Copy)]
struct ProbeOutcome {
    state: ShadowCapabilityState,
    reason_code: Option<&'static str>,
}

impl ProbeOutcome {
    const fn available() -> Self {
        Self {
            state: ShadowCapabilityState::ObservedAvailable,
            reason_code: None,
        }
    }

    const fn unavailable(reason_code: &'static str) -> Self {
        Self {
            state: ShadowCapabilityState::ObservedUnavailable,
            reason_code: Some(reason_code),
        }
    }

    const fn unknown(reason_code: &'static str) -> Self {
        Self {
            state: ShadowCapabilityState::Unknown,
            reason_code: Some(reason_code),
        }
    }
}

fn probe_basic_fts5(connection: &Connection) -> ProbeOutcome {
    if connection
        .execute_batch(
            "CREATE VIRTUAL TABLE probe_fts5 USING fts5(body);\n\
             INSERT INTO probe_fts5(body) VALUES ('alpha beta');",
        )
        .is_err()
    {
        return ProbeOutcome::unavailable("FTS5_MODULE_UNAVAILABLE");
    }
    match match_count(
        connection,
        "SELECT count(*) FROM probe_fts5 WHERE probe_fts5 MATCH ?1",
        "alpha",
    ) {
        Some(1) => ProbeOutcome::available(),
        Some(_) => ProbeOutcome::unknown("FTS5_MATCH_SEMANTICS_UNEXPECTED"),
        None => ProbeOutcome::unknown("FTS5_MATCH_QUERY_FAILED"),
    }
}

fn probe_unicode61(connection: &Connection) -> ProbeOutcome {
    if connection
        .execute_batch(
            "CREATE VIRTUAL TABLE probe_unicode61 USING fts5(body, tokenize='unicode61');\n\
             INSERT INTO probe_unicode61(body) VALUES ('alpha beta');",
        )
        .is_err()
    {
        return ProbeOutcome::unavailable("FTS5_UNICODE61_UNAVAILABLE");
    }
    match match_count(
        connection,
        "SELECT count(*) FROM probe_unicode61 WHERE probe_unicode61 MATCH ?1",
        "alpha",
    ) {
        Some(1) => ProbeOutcome::available(),
        Some(_) => ProbeOutcome::unknown("FTS5_UNICODE61_SEMANTICS_UNEXPECTED"),
        None => ProbeOutcome::unknown("FTS5_UNICODE61_QUERY_FAILED"),
    }
}

fn probe_trigram(connection: &Connection) -> ProbeOutcome {
    if connection
        .execute_batch(
            "CREATE VIRTUAL TABLE probe_trigram USING fts5(body, tokenize='trigram');\n\
             INSERT INTO probe_trigram(body) VALUES ('abcdefghij');",
        )
        .is_err()
    {
        return ProbeOutcome::unavailable("FTS5_TRIGRAM_UNAVAILABLE");
    }
    match match_count(
        connection,
        "SELECT count(*) FROM probe_trigram WHERE probe_trigram MATCH ?1",
        "cdef",
    ) {
        Some(1) => ProbeOutcome::available(),
        Some(_) => ProbeOutcome::unknown("FTS5_TRIGRAM_SEMANTICS_UNEXPECTED"),
        None => ProbeOutcome::unknown("FTS5_TRIGRAM_QUERY_FAILED"),
    }
}

fn probe_contentless(connection: &Connection) -> ProbeOutcome {
    if connection
        .execute_batch(
            "CREATE VIRTUAL TABLE probe_contentless USING fts5(body, content='');\n\
             INSERT INTO probe_contentless(rowid, body) VALUES (1, 'alpha beta');",
        )
        .is_err()
    {
        return ProbeOutcome::unavailable("FTS5_CONTENTLESS_UNAVAILABLE");
    }
    match match_count(
        connection,
        "SELECT count(*) FROM probe_contentless WHERE probe_contentless MATCH ?1",
        "alpha",
    ) {
        Some(1) => ProbeOutcome::available(),
        Some(_) => ProbeOutcome::unknown("FTS5_CONTENTLESS_SEMANTICS_UNEXPECTED"),
        None => ProbeOutcome::unknown("FTS5_CONTENTLESS_QUERY_FAILED"),
    }
}

fn probe_contentless_delete(connection: &Connection) -> ProbeOutcome {
    if connection
        .execute_batch(
            "CREATE VIRTUAL TABLE probe_contentless_delete USING fts5(\
                 body, content='', contentless_delete=1\
             );\n\
             INSERT INTO probe_contentless_delete(rowid, body) VALUES (1, 'alpha beta');\n\
             DELETE FROM probe_contentless_delete WHERE rowid = 1;",
        )
        .is_err()
    {
        return ProbeOutcome::unavailable("FTS5_CONTENTLESS_DELETE_UNAVAILABLE");
    }
    match match_count(
        connection,
        "SELECT count(*) FROM probe_contentless_delete WHERE probe_contentless_delete MATCH ?1",
        "alpha",
    ) {
        Some(0) => ProbeOutcome::available(),
        Some(_) => ProbeOutcome::unknown("FTS5_CONTENTLESS_DELETE_SEMANTICS_UNEXPECTED"),
        None => ProbeOutcome::unknown("FTS5_CONTENTLESS_DELETE_QUERY_FAILED"),
    }
}

fn probe_integrity_check(connection: &Connection) -> ProbeOutcome {
    if connection
        .execute_batch(
            "CREATE VIRTUAL TABLE probe_integrity USING fts5(body);\n\
             INSERT INTO probe_integrity(body) VALUES ('alpha beta');",
        )
        .is_err()
    {
        return ProbeOutcome::unavailable("FTS5_INTEGRITY_TABLE_UNAVAILABLE");
    }
    if connection
        .execute_batch("INSERT INTO probe_integrity(probe_integrity) VALUES ('integrity-check');")
        .is_ok()
    {
        ProbeOutcome::available()
    } else {
        ProbeOutcome::unknown("FTS5_INTEGRITY_CHECK_FAILED")
    }
}

fn match_count(connection: &Connection, sql: &str, query: &str) -> Option<i64> {
    connection
        .query_row(sql, [query], |row| row.get::<_, i64>(0))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_sqlite_probe_observes_required_candidate_capabilities() {
        let report = probe_sqlite_fts5_capabilities();

        assert_eq!(
            report.schema_version,
            SHADOW_SQLITE_CAPABILITY_SCHEMA_VERSION
        );
        assert!(
            report
                .sqlite_version
                .as_deref()
                .is_some_and(|v| !v.is_empty())
        );
        assert_eq!(
            report.compile_option_fts5,
            ShadowCapabilityState::ObservedAvailable
        );
        assert_eq!(report.fts5, ShadowCapabilityState::ObservedAvailable);
        assert_eq!(report.unicode61, ShadowCapabilityState::ObservedAvailable);
        assert_eq!(report.trigram, ShadowCapabilityState::ObservedAvailable);
        assert_eq!(report.contentless, ShadowCapabilityState::ObservedAvailable);
        assert_eq!(
            report.contentless_delete,
            ShadowCapabilityState::ObservedAvailable
        );
        assert_eq!(
            report.integrity_check,
            ShadowCapabilityState::ObservedAvailable
        );
        assert!(!report.extension_loading_attempted);
        assert_eq!(report.fallback_backend_id, DIRECT_SCAN_FALLBACK_BACKEND_ID);
        assert!(report.reason_codes.is_empty());
    }

    #[test]
    fn runtime_failure_report_fails_closed_to_unknown_and_keeps_direct_scan() {
        let report = unavailable_runtime_report("SYNTHETIC_PROBE_FAILURE");
        assert_eq!(report.sqlite_version, None);
        assert_eq!(report.fts5, ShadowCapabilityState::Unknown);
        assert_eq!(report.unicode61, ShadowCapabilityState::Unknown);
        assert_eq!(report.trigram, ShadowCapabilityState::Unknown);
        assert!(!report.extension_loading_attempted);
        assert_eq!(report.fallback_backend_id, DIRECT_SCAN_FALLBACK_BACKEND_ID);
        assert_eq!(report.reason_codes, vec!["SYNTHETIC_PROBE_FAILURE"]);
    }

    #[test]
    fn capability_state_serialization_matches_the_frozen_benchmark_labels() {
        let cases = [
            (
                ShadowCapabilityState::ObservedAvailable,
                "\"OBSERVED_AVAILABLE\"",
            ),
            (
                ShadowCapabilityState::ObservedUnavailable,
                "\"OBSERVED_UNAVAILABLE\"",
            ),
            (ShadowCapabilityState::NotApplicable, "\"NOT_APPLICABLE\""),
            (ShadowCapabilityState::Unknown, "\"UNKNOWN\""),
        ];
        for (state, expected) in cases {
            assert_eq!(
                serde_json::to_string(&state).expect("serialize state"),
                expected
            );
        }
    }
}
