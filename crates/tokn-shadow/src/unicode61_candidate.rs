use anyhow::{Context, Result, bail};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use tokn_domain::scoped_shadow_document_id_bytes;
use tokn_platform::{
    DirectScanDocument, DirectScanManifest, direct_scan_query_terms, verify_direct_scan_document,
    verify_direct_scan_manifest, visit_verified_direct_scan_documents,
};

use crate::{ShadowCapabilityState, probe_sqlite_fts5_capabilities};

pub const SQLITE_FTS5_UNICODE61_BACKEND_ID: &str = "SQLITE_FTS5_UNICODE61_V0";
pub const SQLITE_FTS5_UNICODE61_TOKENIZER_ID: &str = "unicode61";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowIndexedDocument {
    pub shadow_document_id: String,
    pub relative_path: String,
    pub source_stable_id: String,
    pub index_content_hash: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqliteFts5Unicode61Hit {
    pub shadow_document_id: String,
    pub relative_path: String,
    pub source_stable_id: String,
    pub index_content_hash: String,
    pub bytes: u64,
    /// SQLite FTS5 BM25 ordering signal. Lower values rank first.
    /// This is not semantic confidence or evidence of context value.
    pub backend_score: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SqliteFts5Unicode61QueryReport {
    pub backend_id: String,
    pub tokenizer_id: String,
    pub total_matching_files: u64,
    pub hits: Vec<SqliteFts5Unicode61Hit>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqliteFts5Unicode61BuildReport {
    pub backend_id: String,
    pub tokenizer_id: String,
    pub documents_indexed: u64,
    pub source_bytes_read: u64,
    pub persistent_index: bool,
    pub contentless: bool,
}

pub struct SqliteFts5Unicode61Candidate {
    connection: Connection,
    manifest: DirectScanManifest,
    documents: Vec<ShadowIndexedDocument>,
    build_report: SqliteFts5Unicode61BuildReport,
}

impl SqliteFts5Unicode61Candidate {
    pub fn build(manifest: DirectScanManifest, project_scope_key: &str) -> Result<Self> {
        if project_scope_key.is_empty() {
            bail!("SQLITE_FTS5_UNICODE61_V0 project scope key must not be empty");
        }
        let capability = probe_sqlite_fts5_capabilities();
        for (name, state) in [
            ("FTS5", capability.fts5),
            ("unicode61", capability.unicode61),
            ("contentless", capability.contentless),
            ("contentless-delete", capability.contentless_delete),
        ] {
            if state != ShadowCapabilityState::ObservedAvailable {
                bail!("SQLITE_FTS5_UNICODE61_V0 required capability unavailable: {name}");
            }
        }

        let verification = verify_direct_scan_manifest(&manifest, project_scope_key)?;

        let connection = Connection::open_in_memory()
            .context("SQLITE_FTS5_UNICODE61_V0 cannot open in-memory SQLite")?;
        connection
            .execute_batch(
                "CREATE VIRTUAL TABLE shadow_unicode61 USING fts5(\
                     path, body, content='', contentless_delete=1, tokenize='unicode61'\
                 );",
            )
            .context("SQLITE_FTS5_UNICODE61_V0 cannot create contentless FTS5 table")?;

        let mut documents = Vec::with_capacity(manifest.documents.len());
        let mut rowid = 1_i64;
        let source_bytes_read = visit_verified_direct_scan_documents(
            &manifest,
            project_scope_key,
            |document, text| {
                connection
                    .execute(
                        "INSERT INTO shadow_unicode61(rowid, path, body) VALUES (?1, ?2, ?3)",
                        params![rowid, document.relative_path, text],
                    )
                    .context("SQLITE_FTS5_UNICODE61_V0 cannot insert document")?;
                documents.push(ShadowIndexedDocument {
                    shadow_document_id: scoped_shadow_document_id_bytes(
                        project_scope_key,
                        &document.source_stable_id,
                        &document.index_content_hash,
                    ),
                    relative_path: document.relative_path.clone(),
                    source_stable_id: document.source_stable_id.clone(),
                    index_content_hash: document.index_content_hash.clone(),
                    bytes: document.bytes,
                });
                rowid = rowid
                    .checked_add(1)
                    .context("SQLITE_FTS5_UNICODE61_V0 rowid overflow")?;
                Ok(())
            },
        )?;

        connection
            .execute_batch(
                "INSERT INTO shadow_unicode61(shadow_unicode61) VALUES ('integrity-check');",
            )
            .context("SQLITE_FTS5_UNICODE61_V0 integrity-check failed after build")?;

        let build_report = SqliteFts5Unicode61BuildReport {
            backend_id: SQLITE_FTS5_UNICODE61_BACKEND_ID.to_owned(),
            tokenizer_id: SQLITE_FTS5_UNICODE61_TOKENIZER_ID.to_owned(),
            documents_indexed: documents.len() as u64,
            source_bytes_read: verification
                .eligible_bytes
                .saturating_add(source_bytes_read),
            persistent_index: false,
            contentless: true,
        };

        Ok(Self {
            connection,
            manifest,
            documents,
            build_report,
        })
    }

    pub fn build_report(&self) -> &SqliteFts5Unicode61BuildReport {
        &self.build_report
    }

    pub fn documents(&self) -> &[ShadowIndexedDocument] {
        &self.documents
    }

    pub fn query(
        &self,
        project_scope_key: &str,
        query: &str,
        limit: usize,
    ) -> Result<SqliteFts5Unicode61QueryReport> {
        verify_direct_scan_manifest(&self.manifest, project_scope_key)?;
        if limit == 0 {
            bail!("SQLITE_FTS5_UNICODE61_V0 result limit must be at least 1");
        }
        let terms = direct_scan_query_terms(query)?;
        let expression = fts5_all_terms_expression(&terms);
        let limit = i64::try_from(limit)
            .context("SQLITE_FTS5_UNICODE61_V0 result limit exceeds SQLite range")?;

        let total_matching_files: i64 = self
            .connection
            .query_row(
                "SELECT count(*) FROM shadow_unicode61 WHERE shadow_unicode61 MATCH ?1",
                [&expression],
                |row| row.get(0),
            )
            .context("SQLITE_FTS5_UNICODE61_V0 count query failed")?;

        let mut statement = self
            .connection
            .prepare(
                "SELECT rowid, bm25(shadow_unicode61) AS score \
                 FROM shadow_unicode61 \
                 WHERE shadow_unicode61 MATCH ?1 \
                 ORDER BY score ASC, rowid ASC \
                 LIMIT ?2",
            )
            .context("SQLITE_FTS5_UNICODE61_V0 query prepare failed")?;
        let rows = statement
            .query_map(params![expression, limit], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, f64>(1)?))
            })
            .context("SQLITE_FTS5_UNICODE61_V0 query execution failed")?;

        let mut hits = Vec::new();
        for row in rows {
            let (rowid, backend_score) =
                row.context("SQLITE_FTS5_UNICODE61_V0 query row decode failed")?;
            let index = usize::try_from(rowid - 1)
                .context("SQLITE_FTS5_UNICODE61_V0 returned invalid rowid")?;
            let document = self
                .documents
                .get(index)
                .context("SQLITE_FTS5_UNICODE61_V0 returned unknown rowid")?;
            hits.push(SqliteFts5Unicode61Hit {
                shadow_document_id: document.shadow_document_id.clone(),
                relative_path: document.relative_path.clone(),
                source_stable_id: document.source_stable_id.clone(),
                index_content_hash: document.index_content_hash.clone(),
                bytes: document.bytes,
                backend_score,
            });
        }

        Ok(SqliteFts5Unicode61QueryReport {
            backend_id: SQLITE_FTS5_UNICODE61_BACKEND_ID.to_owned(),
            tokenizer_id: SQLITE_FTS5_UNICODE61_TOKENIZER_ID.to_owned(),
            total_matching_files: u64::try_from(total_matching_files)
                .context("SQLITE_FTS5_UNICODE61_V0 negative match count")?,
            hits,
        })
    }

    pub fn verify_hit_current(
        &self,
        project_scope_key: &str,
        hit: &SqliteFts5Unicode61Hit,
    ) -> Result<u64> {
        let metadata = self
            .documents
            .iter()
            .find(|document| document.shadow_document_id == hit.shadow_document_id)
            .context("SQLITE_FTS5_UNICODE61_V0 hit is not part of this candidate")?;
        if metadata.relative_path != hit.relative_path
            || metadata.source_stable_id != hit.source_stable_id
            || metadata.index_content_hash != hit.index_content_hash
        {
            bail!("SQLITE_FTS5_UNICODE61_V0 hit metadata mismatch");
        }
        let manifest_document = self
            .manifest
            .documents
            .iter()
            .find(|document| same_document_identity(document, metadata))
            .context("SQLITE_FTS5_UNICODE61_V0 manifest document missing")?;
        verify_direct_scan_document(&self.manifest, project_scope_key, manifest_document)
    }
}

fn same_document_identity(document: &DirectScanDocument, indexed: &ShadowIndexedDocument) -> bool {
    document.relative_path == indexed.relative_path
        && document.source_stable_id == indexed.source_stable_id
        && document.index_content_hash == indexed.index_content_hash
        && document.bytes == indexed.bytes
}

fn fts5_all_terms_expression(terms: &[String]) -> String {
    terms
        .iter()
        .map(|term| format!("\"{}\"", term.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" AND ")
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };

    use tokn_platform::{DirectScanPolicy, build_direct_scan_manifest, query_direct_scan_manifest};

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new(name: &str) -> Self {
            let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "tokn-shadow-unicode61-{name}-{}-{counter}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(path.join("src")).expect("create src");
            fs::create_dir_all(path.join("docs")).expect("create docs");
            assert!(
                Command::new("git")
                    .arg("init")
                    .arg("--quiet")
                    .arg(&path)
                    .status()
                    .expect("git init")
                    .success()
            );
            Self { path }
        }

        fn write(&self, relative: &str, bytes: &[u8]) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, bytes).expect("write fixture");
        }

        fn add(&self, paths: &[&str]) {
            assert!(
                Command::new("git")
                    .arg("-C")
                    .arg(&self.path)
                    .arg("add")
                    .arg("--")
                    .args(paths)
                    .status()
                    .expect("git add")
                    .success()
            );
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn fixture() -> (TempRepo, DirectScanManifest) {
        let repo = TempRepo::new("fixture");
        repo.write(".gitignore", b"ignored.txt\n");
        repo.write("src/alpha.rs", b"pub fn AlphaWidget() {}\n");
        repo.write("src/beta.rs", b"pub fn BetaThing() {}\n");
        repo.write("docs/guide.md", b"Alpha widget guide for operators.\n");
        repo.write("ignored.txt", b"alpha widget hidden\n");
        repo.add(&[".gitignore", "src/alpha.rs", "src/beta.rs"]);
        let manifest = build_direct_scan_manifest(
            &repo.path,
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 1024,
                max_files: 32,
            },
        )
        .expect("build direct manifest");
        (repo, manifest)
    }

    #[test]
    fn build_uses_same_direct_scan_corpus_and_retains_no_source_text_metadata() {
        let (_repo, manifest) = fixture();
        let expected_documents = manifest.eligible_files();
        let expected_bytes = manifest.eligible_bytes();
        let candidate =
            SqliteFts5Unicode61Candidate::build(manifest, "project-a").expect("candidate build");

        assert_eq!(
            candidate.build_report().documents_indexed,
            expected_documents
        );
        assert_eq!(
            candidate.build_report().source_bytes_read,
            expected_bytes.saturating_mul(2)
        );
        assert!(!candidate.build_report().persistent_index);
        assert!(candidate.build_report().contentless);
        assert!(
            candidate
                .documents()
                .iter()
                .all(|document| document.shadow_document_id.starts_with("sdoc-v1-"))
        );
    }

    #[test]
    fn unicode61_quality_difference_is_explicit_for_camelcase_substrings() {
        let (_repo, manifest) = fixture();
        let reference = query_direct_scan_manifest(&manifest, "project-a", "alpha widget", 10)
            .expect("direct scan reference query");
        assert_eq!(reference.total_matching_files, 2);

        let candidate =
            SqliteFts5Unicode61Candidate::build(manifest, "project-a").expect("candidate build");
        let report = candidate
            .query("project-a", "alpha widget", 10)
            .expect("query");

        assert_eq!(report.backend_id, SQLITE_FTS5_UNICODE61_BACKEND_ID);
        assert_eq!(report.tokenizer_id, SQLITE_FTS5_UNICODE61_TOKENIZER_ID);
        assert_eq!(report.total_matching_files, 1);
        assert_eq!(report.hits.len(), 1);
        assert_eq!(report.hits[0].relative_path, "docs/guide.md");

        let exact_identifier = candidate
            .query("project-a", "AlphaWidget", 10)
            .expect("exact identifier query");
        assert_eq!(exact_identifier.total_matching_files, 1);
        assert_eq!(exact_identifier.hits[0].relative_path, "src/alpha.rs");
    }

    #[test]
    fn current_hit_verification_fails_closed_after_content_mutation() {
        let (repo, manifest) = fixture();
        let candidate =
            SqliteFts5Unicode61Candidate::build(manifest, "project-a").expect("candidate build");
        let report = candidate.query("project-a", "BetaThing", 5).expect("query");
        let hit = report.hits.first().expect("beta hit");
        assert!(candidate.verify_hit_current("project-a", hit).is_ok());

        repo.write("src/beta.rs", b"pub fn ChangedThing() {}\n");
        let error = candidate
            .verify_hit_current("project-a", hit)
            .expect_err("mutated hit must require refresh");
        assert!(error.to_string().contains("REFRESH_REQUIRED"));
        assert!(!error.to_string().contains("src/beta.rs"));
    }

    #[test]
    fn corpus_addition_after_manifest_is_rejected_during_candidate_build() {
        let (repo, manifest) = fixture();
        repo.write("new-untracked.rs", b"pub fn NewUntracked() {}\n");
        let error = match SqliteFts5Unicode61Candidate::build(manifest, "project-a") {
            Ok(_) => panic!("changed corpus must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("REFRESH_REQUIRED"));
    }

    #[test]
    fn query_rejects_corpus_addition_after_build_before_returning_false_negative() {
        let (repo, manifest) = fixture();
        let candidate =
            SqliteFts5Unicode61Candidate::build(manifest, "project-a").expect("candidate build");
        repo.write("late-untracked.rs", b"pub fn LateOnlyIdentifier() {}\n");

        let error = candidate
            .query("project-a", "LateOnlyIdentifier", 10)
            .expect_err("changed corpus must require refresh before query");
        assert!(error.to_string().contains("REFRESH_REQUIRED"));
        assert!(!error.to_string().contains("late-untracked.rs"));
    }

    #[test]
    fn stale_manifest_is_rejected_during_candidate_build() {
        let (repo, manifest) = fixture();
        repo.write("src/alpha.rs", b"pub fn ChangedBeforeBuild() {}\n");
        let error = match SqliteFts5Unicode61Candidate::build(manifest, "project-a") {
            Ok(_) => panic!("stale manifest must fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("REFRESH_REQUIRED"));
    }

    #[test]
    fn fts_query_escapes_quotes_and_operators_as_literal_terms() {
        let (_repo, manifest) = fixture();
        let candidate =
            SqliteFts5Unicode61Candidate::build(manifest, "project-a").expect("candidate build");
        let report = candidate
            .query("project-a", "alpha \" OR beta", 10)
            .expect("escaped query must remain valid");
        assert_eq!(report.total_matching_files, 0);
    }
}
