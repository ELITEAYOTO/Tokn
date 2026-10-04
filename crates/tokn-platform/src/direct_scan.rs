use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use tokn_domain::{scoped_index_content_hash_bytes, scoped_source_id_bytes};

pub const DIRECT_SCAN_QUERY_MAX_BYTES: usize = 4096;
pub const DIRECT_SCAN_QUERY_MAX_TERMS: usize = 64;
pub const DIRECT_SCAN_MAX_FILE_BYTES_HARD_LIMIT: u64 = 16 * 1024 * 1024;
pub const DIRECT_SCAN_MAX_FILES_HARD_LIMIT: usize = 100_000;
pub const DIRECT_SCAN_MAX_GIT_OUTPUT_BYTES: u64 = 16 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirectScanPolicy {
    pub max_file_bytes: u64,
    pub max_files: usize,
}

impl DirectScanPolicy {
    pub fn validate(self) -> Result<()> {
        if self.max_file_bytes == 0 || self.max_file_bytes > DIRECT_SCAN_MAX_FILE_BYTES_HARD_LIMIT {
            bail!("DIRECT_SCAN_V0 max_file_bytes exceeds the operational safety limit");
        }
        if self.max_files == 0 || self.max_files > DIRECT_SCAN_MAX_FILES_HARD_LIMIT {
            bail!("DIRECT_SCAN_V0 max_files exceeds the operational safety limit");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectScanSkipReason {
    UnsafePath,
    Missing,
    OutsideRoot,
    NotRegularFile,
    TooLarge,
    Binary,
    NonUtf8,
    Unreadable,
}

impl DirectScanSkipReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsafePath => "UNSAFE_PATH",
            Self::Missing => "MISSING",
            Self::OutsideRoot => "OUTSIDE_ROOT",
            Self::NotRegularFile => "NOT_REGULAR_FILE",
            Self::TooLarge => "TOO_LARGE",
            Self::Binary => "BINARY",
            Self::NonUtf8 => "NON_UTF8",
            Self::Unreadable => "UNREADABLE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectScanSkippedDocument {
    pub relative_path: String,
    pub reason: DirectScanSkipReason,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectScanDocument {
    pub relative_path: String,
    pub source_stable_id: String,
    pub index_content_hash: String,
    pub bytes: u64,
}

#[derive(Clone)]
pub struct DirectScanManifest {
    root: PathBuf,
    policy: DirectScanPolicy,
    pub discovered_files: u64,
    pub documents: Vec<DirectScanDocument>,
    pub skipped: Vec<DirectScanSkippedDocument>,
}

impl DirectScanManifest {
    pub fn eligible_files(&self) -> u64 {
        self.documents.len() as u64
    }

    pub fn skipped_files(&self) -> u64 {
        self.skipped.len() as u64
    }

    pub fn eligible_bytes(&self) -> u64 {
        self.documents.iter().fold(0_u64, |total, document| {
            total.saturating_add(document.bytes)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectScanHit {
    pub relative_path: String,
    pub source_stable_id: String,
    pub index_content_hash: String,
    pub bytes: u64,
    pub path_term_hits: u32,
    pub content_term_hits: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectScanQueryReport {
    pub files_examined: u64,
    pub source_bytes_read: u64,
    pub total_matching_files: u64,
    pub hits: Vec<DirectScanHit>,
}

pub fn build_direct_scan_manifest(
    repository_root: &Path,
    project_scope_key: &str,
    policy: DirectScanPolicy,
) -> Result<DirectScanManifest> {
    policy.validate()?;
    if project_scope_key.is_empty() {
        bail!("DIRECT_SCAN_V0 project scope key must not be empty");
    }

    let root = canonical_repository_root(repository_root)?;
    let candidates = git_file_candidates(&root, policy.max_files)?;
    let mut documents = Vec::new();
    let mut skipped = Vec::new();

    for relative_path in &candidates {
        let read = match read_candidate(&root, relative_path, policy) {
            Ok(read) => read,
            Err(reason) => {
                skipped.push(DirectScanSkippedDocument {
                    relative_path: relative_path.clone(),
                    reason,
                });
                continue;
            }
        };
        let Some(locator) = source_locator_from_git_relative_path(relative_path) else {
            skipped.push(DirectScanSkippedDocument {
                relative_path: relative_path.clone(),
                reason: DirectScanSkipReason::UnsafePath,
            });
            continue;
        };

        documents.push(DirectScanDocument {
            relative_path: relative_path.clone(),
            source_stable_id: scoped_source_id_bytes(project_scope_key, locator.as_bytes()),
            index_content_hash: scoped_index_content_hash_bytes(project_scope_key, &read.bytes),
            bytes: read.bytes.len() as u64,
        });
    }

    documents.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    skipped.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(DirectScanManifest {
        root,
        policy,
        discovered_files: candidates.len() as u64,
        documents,
        skipped,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectScanManifestVerification {
    pub discovered_files: u64,
    pub eligible_files: u64,
    pub skipped_files: u64,
    pub eligible_bytes: u64,
}

pub fn verify_direct_scan_manifest(
    manifest: &DirectScanManifest,
    project_scope_key: &str,
) -> Result<DirectScanManifestVerification> {
    if project_scope_key.is_empty() {
        bail!("DIRECT_SCAN_V0 project scope key must not be empty");
    }
    let current = build_direct_scan_manifest(&manifest.root, project_scope_key, manifest.policy)?;
    if current.discovered_files != manifest.discovered_files
        || current.documents != manifest.documents
        || current.skipped != manifest.skipped
    {
        bail!("DIRECT_SCAN_V0 manifest verification failed: REFRESH_REQUIRED");
    }
    Ok(DirectScanManifestVerification {
        discovered_files: current.discovered_files,
        eligible_files: current.eligible_files(),
        skipped_files: current.skipped_files(),
        eligible_bytes: current.eligible_bytes(),
    })
}

pub fn verify_direct_scan_document(
    manifest: &DirectScanManifest,
    project_scope_key: &str,
    document: &DirectScanDocument,
) -> Result<u64> {
    if project_scope_key.is_empty() {
        bail!("DIRECT_SCAN_V0 project scope key must not be empty");
    }
    if !manifest
        .documents
        .iter()
        .any(|candidate| candidate == document)
    {
        bail!("DIRECT_SCAN_V0 document is not part of the manifest");
    }
    let current = read_candidate(&manifest.root, &document.relative_path, manifest.policy)
        .map_err(|reason| {
            anyhow::anyhow!(
                "DIRECT_SCAN_V0 manifest verification failed for {}: {}",
                document.source_stable_id,
                reason.as_str()
            )
        })?;
    let current_hash = scoped_index_content_hash_bytes(project_scope_key, &current.bytes);
    if current_hash != document.index_content_hash {
        bail!(
            "DIRECT_SCAN_V0 manifest verification failed for {}: REFRESH_REQUIRED",
            document.source_stable_id
        );
    }
    Ok(current.bytes.len() as u64)
}

pub fn visit_verified_direct_scan_documents<F>(
    manifest: &DirectScanManifest,
    project_scope_key: &str,
    mut visitor: F,
) -> Result<u64>
where
    F: FnMut(&DirectScanDocument, &str) -> Result<()>,
{
    if project_scope_key.is_empty() {
        bail!("DIRECT_SCAN_V0 project scope key must not be empty");
    }
    let mut source_bytes_read = 0_u64;
    for document in &manifest.documents {
        let current = read_candidate(&manifest.root, &document.relative_path, manifest.policy)
            .map_err(|reason| {
                anyhow::anyhow!(
                    "DIRECT_SCAN_V0 manifest verification failed for {}: {}",
                    document.source_stable_id,
                    reason.as_str()
                )
            })?;
        let current_hash = scoped_index_content_hash_bytes(project_scope_key, &current.bytes);
        if current_hash != document.index_content_hash {
            bail!(
                "DIRECT_SCAN_V0 manifest verification failed for {}: REFRESH_REQUIRED",
                document.source_stable_id
            );
        }
        let text = std::str::from_utf8(&current.bytes)
            .context("DIRECT_SCAN_V0 verified document unexpectedly became non-UTF-8")?;
        visitor(document, text)?;
        source_bytes_read = source_bytes_read.saturating_add(current.bytes.len() as u64);
    }
    Ok(source_bytes_read)
}

pub fn direct_scan_query_terms(query: &str) -> Result<Vec<String>> {
    normalized_query_terms(query)
}

pub fn query_direct_scan_manifest(
    manifest: &DirectScanManifest,
    project_scope_key: &str,
    query: &str,
    limit: usize,
) -> Result<DirectScanQueryReport> {
    if project_scope_key.is_empty() {
        bail!("DIRECT_SCAN_V0 project scope key must not be empty");
    }
    if limit == 0 {
        bail!("DIRECT_SCAN_V0 result limit must be at least 1");
    }
    let terms = direct_scan_query_terms(query)?;
    let mut hits = Vec::new();
    let mut files_examined = 0_u64;
    let mut source_bytes_read = 0_u64;

    for document in &manifest.documents {
        let current = read_candidate(&manifest.root, &document.relative_path, manifest.policy)
            .map_err(|reason| {
                anyhow::anyhow!(
                    "DIRECT_SCAN_V0 manifest verification failed for {}: {}",
                    document.source_stable_id,
                    reason.as_str()
                )
            })?;
        files_examined += 1;
        source_bytes_read = source_bytes_read.saturating_add(current.bytes.len() as u64);

        let current_hash = scoped_index_content_hash_bytes(project_scope_key, &current.bytes);
        if current_hash != document.index_content_hash {
            bail!(
                "DIRECT_SCAN_V0 manifest verification failed for {}: REFRESH_REQUIRED",
                document.source_stable_id
            );
        }

        let path = document.relative_path.to_lowercase();
        let content = std::str::from_utf8(&current.bytes)
            .expect("DIRECT_SCAN_V0 read_candidate validates UTF-8")
            .to_lowercase();
        let path_term_hits = terms
            .iter()
            .filter(|term| path.contains(term.as_str()))
            .count();
        let content_term_hits = terms
            .iter()
            .filter(|term| content.contains(term.as_str()))
            .count();
        let matches_all = terms
            .iter()
            .all(|term| path.contains(term.as_str()) || content.contains(term.as_str()));
        if !matches_all {
            continue;
        }

        hits.push(DirectScanHit {
            relative_path: document.relative_path.clone(),
            source_stable_id: document.source_stable_id.clone(),
            index_content_hash: document.index_content_hash.clone(),
            bytes: document.bytes,
            path_term_hits: path_term_hits as u32,
            content_term_hits: content_term_hits as u32,
        });
    }

    hits.sort_by(|left, right| {
        right
            .path_term_hits
            .cmp(&left.path_term_hits)
            .then_with(|| right.content_term_hits.cmp(&left.content_term_hits))
            .then_with(|| left.relative_path.cmp(&right.relative_path))
    });
    let total_matching_files = hits.len() as u64;
    hits.truncate(limit);

    Ok(DirectScanQueryReport {
        files_examined,
        source_bytes_read,
        total_matching_files,
        hits,
    })
}

#[derive(Debug)]
struct CandidateRead {
    bytes: Vec<u8>,
}

fn canonical_repository_root(root: &Path) -> Result<PathBuf> {
    let root =
        fs::canonicalize(root).context("cannot canonicalize DIRECT_SCAN_V0 repository root")?;
    if !root.is_dir() {
        bail!("DIRECT_SCAN_V0 repository root is not a directory");
    }

    let output = Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("cannot execute git rev-parse for DIRECT_SCAN_V0")?;
    if !output.status.success() {
        bail!("DIRECT_SCAN_V0 requires a Git repository root");
    }
    let top = std::str::from_utf8(&output.stdout)
        .context("DIRECT_SCAN_V0 Git repository root output is not UTF-8")?
        .trim();
    let top = fs::canonicalize(top).context("cannot canonicalize Git top-level directory")?;
    if top != root {
        bail!("DIRECT_SCAN_V0 repository_root must be the Git top-level directory");
    }
    Ok(root)
}

fn git_file_candidates(root: &Path, max_files: usize) -> Result<Vec<String>> {
    let mut child = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
            "--full-name",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("cannot execute git ls-files for DIRECT_SCAN_V0")?;
    let stdout = child
        .stdout
        .take()
        .context("DIRECT_SCAN_V0 git ls-files stdout unavailable")?;
    let mut raw = Vec::new();
    stdout
        .take(DIRECT_SCAN_MAX_GIT_OUTPUT_BYTES + 1)
        .read_to_end(&mut raw)
        .context("cannot read bounded git ls-files output")?;
    if raw.len() as u64 > DIRECT_SCAN_MAX_GIT_OUTPUT_BYTES {
        let _ = child.kill();
        let _ = child.wait();
        bail!("DIRECT_SCAN_V0 git file list exceeds the operational byte limit");
    }
    let status = child.wait().context("cannot wait for git ls-files")?;
    if !status.success() {
        bail!("DIRECT_SCAN_V0 git ls-files failed");
    }
    let raw =
        std::str::from_utf8(&raw).context("DIRECT_SCAN_V0 git ls-files output is not UTF-8")?;
    let mut candidates = BTreeSet::new();
    for relative in raw.split('\0').filter(|value| !value.is_empty()) {
        candidates.insert(relative.to_string());
        if candidates.len() > max_files {
            bail!("DIRECT_SCAN_V0 discovered file count exceeds the configured limit");
        }
    }
    Ok(candidates.into_iter().collect())
}

fn read_candidate(
    root: &Path,
    relative_path: &str,
    policy: DirectScanPolicy,
) -> std::result::Result<CandidateRead, DirectScanSkipReason> {
    if !is_safe_git_relative_path(relative_path) {
        return Err(DirectScanSkipReason::UnsafePath);
    }
    let candidate = root.join(relative_path);
    let canonical = match fs::canonicalize(&candidate) {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(DirectScanSkipReason::Missing);
        }
        Err(_) => return Err(DirectScanSkipReason::Unreadable),
    };
    if !canonical.starts_with(root) {
        return Err(DirectScanSkipReason::OutsideRoot);
    }
    let metadata = fs::metadata(&canonical).map_err(|_| DirectScanSkipReason::Unreadable)?;
    if !metadata.is_file() {
        return Err(DirectScanSkipReason::NotRegularFile);
    }
    if metadata.len() > policy.max_file_bytes {
        return Err(DirectScanSkipReason::TooLarge);
    }

    let file = File::open(&canonical).map_err(|_| DirectScanSkipReason::Unreadable)?;
    let initial_capacity = metadata.len().min(policy.max_file_bytes).min(64 * 1024) as usize;
    let mut bytes = Vec::with_capacity(initial_capacity);
    file.take(policy.max_file_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| DirectScanSkipReason::Unreadable)?;
    if bytes.len() as u64 > policy.max_file_bytes {
        return Err(DirectScanSkipReason::TooLarge);
    }
    if bytes.contains(&0) {
        return Err(DirectScanSkipReason::Binary);
    }
    std::str::from_utf8(&bytes).map_err(|_| DirectScanSkipReason::NonUtf8)?;
    Ok(CandidateRead { bytes })
}

fn is_safe_git_relative_path(relative_path: &str) -> bool {
    if relative_path.is_empty()
        || relative_path.contains('\\')
        || relative_path.chars().any(char::is_control)
        || relative_path.starts_with("//")
        || (relative_path.len() >= 2 && relative_path.as_bytes()[1] == b':')
    {
        return false;
    }
    Path::new(relative_path)
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
}

fn source_locator_from_git_relative_path(relative_path: &str) -> Option<String> {
    if !is_safe_git_relative_path(relative_path) {
        return None;
    }
    let normalized = if cfg!(windows) {
        relative_path.to_ascii_lowercase()
    } else {
        relative_path.to_string()
    };
    Some(format!("file:{normalized}"))
}

fn normalized_query_terms(query: &str) -> Result<Vec<String>> {
    let query = query.trim();
    if query.is_empty() {
        bail!("DIRECT_SCAN_V0 query must not be empty");
    }
    if query.len() > DIRECT_SCAN_QUERY_MAX_BYTES {
        bail!("DIRECT_SCAN_V0 query exceeds the bounded byte limit");
    }
    let mut terms = query
        .to_lowercase()
        .split_whitespace()
        .map(str::to_string)
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    if terms.is_empty() || terms.len() > DIRECT_SCAN_QUERY_MAX_TERMS {
        bail!("DIRECT_SCAN_V0 query term count is outside the bounded limit");
    }
    Ok(terms)
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::*;

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new(name: &str) -> Self {
            let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "tokn-direct-scan-{name}-{}-{counter}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(path.join("src")).expect("create temp repository");
            let status = Command::new("git")
                .arg("init")
                .arg("--quiet")
                .arg(&path)
                .status()
                .expect("run git init");
            assert!(status.success());
            Self { path }
        }

        fn write(&self, relative: &str, bytes: &[u8]) {
            let path = self.path.join(relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create parent");
            }
            fs::write(path, bytes).expect("write fixture file");
        }

        fn git_add(&self, paths: &[&str]) {
            let status = Command::new("git")
                .arg("-C")
                .arg(&self.path)
                .arg("add")
                .arg("--")
                .args(paths)
                .status()
                .expect("git add");
            assert!(status.success());
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn fixture_repo() -> TempRepo {
        let repo = TempRepo::new("fixture");
        repo.write(".gitignore", b"ignored.txt\n");
        repo.write("src/lib.rs", b"pub fn AlphaWidget() {}\n");
        repo.write("README.md", b"Alpha widget documentation\n");
        repo.write("ignored.txt", b"this file must not be discovered\n");
        repo.write("binary.bin", b"abc\0def");
        repo.write("nonutf8.txt", &[0xff, 0xfe]);
        repo.write("oversized.txt", &[b'x'; 128]);
        repo.git_add(&[".gitignore", "src/lib.rs"]);
        repo
    }

    #[test]
    fn manifest_uses_git_corpus_and_explicit_skip_reasons() {
        let repo = fixture_repo();
        let manifest = build_direct_scan_manifest(
            &repo.path,
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 64,
                max_files: 32,
            },
        )
        .expect("build manifest");

        assert_eq!(manifest.discovered_files, 6);
        assert_eq!(manifest.eligible_files(), 3);
        assert_eq!(manifest.skipped_files(), 3);
        assert!(
            manifest
                .documents
                .iter()
                .all(|document| document.relative_path != "ignored.txt")
        );
        assert!(manifest.skipped.iter().any(|entry| {
            entry.relative_path == "binary.bin" && entry.reason == DirectScanSkipReason::Binary
        }));
        assert!(manifest.skipped.iter().any(|entry| {
            entry.relative_path == "nonutf8.txt" && entry.reason == DirectScanSkipReason::NonUtf8
        }));
        assert!(manifest.skipped.iter().any(|entry| {
            entry.relative_path == "oversized.txt" && entry.reason == DirectScanSkipReason::TooLarge
        }));
    }

    #[test]
    fn manifest_source_identity_matches_the_accepted_file_locator_domain() {
        let repo = fixture_repo();
        let manifest = build_direct_scan_manifest(
            &repo.path,
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 64,
                max_files: 32,
            },
        )
        .expect("build manifest");
        let document = manifest
            .documents
            .iter()
            .find(|document| document.relative_path == "src/lib.rs")
            .expect("lib document");

        assert_eq!(
            document.source_stable_id,
            scoped_source_id_bytes("project-a", b"file:src/lib.rs")
        );
        assert!(document.index_content_hash.starts_with("ixc-v1-"));
    }

    #[test]
    fn query_is_bounded_deterministic_and_returns_no_source_text() {
        let repo = fixture_repo();
        let manifest = build_direct_scan_manifest(
            &repo.path,
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 64,
                max_files: 32,
            },
        )
        .expect("build manifest");

        let report = query_direct_scan_manifest(&manifest, "project-a", "alpha widget", 10)
            .expect("query manifest");
        assert_eq!(report.files_examined, 3);
        assert_eq!(report.total_matching_files, 2);
        assert_eq!(report.hits.len(), 2);
        assert_eq!(report.hits[0].relative_path, "README.md");
        assert_eq!(report.hits[1].relative_path, "src/lib.rs");

        let path_rank =
            query_direct_scan_manifest(&manifest, "project-a", "src lib", 1).expect("path query");
        assert_eq!(path_rank.total_matching_files, 1);
        assert_eq!(path_rank.hits[0].relative_path, "src/lib.rs");
        assert_eq!(path_rank.hits[0].path_term_hits, 2);
    }

    #[test]
    fn query_fails_closed_when_manifest_content_changes() {
        let repo = fixture_repo();
        let manifest = build_direct_scan_manifest(
            &repo.path,
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 64,
                max_files: 32,
            },
        )
        .expect("build manifest");
        repo.write("src/lib.rs", b"pub fn ChangedWidget() {}\n");

        let error = query_direct_scan_manifest(&manifest, "project-a", "widget", 10)
            .expect_err("stale manifest must fail closed");
        let message = error.to_string();
        assert!(message.contains("REFRESH_REQUIRED"));
        assert!(!message.contains("src/lib.rs"));
    }

    #[test]
    fn unsafe_relative_paths_are_rejected_before_joining_the_root() {
        for relative in [
            "",
            ".",
            "../escape.rs",
            "/absolute.rs",
            "C:/absolute.rs",
            r"folder\escape.rs",
            "folder/../escape.rs",
        ] {
            assert!(
                !is_safe_git_relative_path(relative),
                "accepted {relative:?}"
            );
        }
        assert!(is_safe_git_relative_path("src/lib.rs"));
    }

    #[test]
    fn nested_directory_is_not_accepted_as_repository_root() {
        let repo = fixture_repo();
        let error = match build_direct_scan_manifest(
            &repo.path.join("src"),
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 64,
                max_files: 32,
            },
        ) {
            Ok(_) => panic!("nested root must fail closed"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("Git top-level"));
    }

    #[test]
    fn operational_policy_limits_are_explicit() {
        assert!(
            DirectScanPolicy {
                max_file_bytes: 0,
                max_files: 1
            }
            .validate()
            .is_err()
        );
        assert!(
            DirectScanPolicy {
                max_file_bytes: DIRECT_SCAN_MAX_FILE_BYTES_HARD_LIMIT + 1,
                max_files: 1
            }
            .validate()
            .is_err()
        );
        assert!(
            DirectScanPolicy {
                max_file_bytes: 1,
                max_files: 0
            }
            .validate()
            .is_err()
        );
        assert!(
            DirectScanPolicy {
                max_file_bytes: 1,
                max_files: DIRECT_SCAN_MAX_FILES_HARD_LIMIT + 1
            }
            .validate()
            .is_err()
        );

        let repo = fixture_repo();
        let result = build_direct_scan_manifest(
            &repo.path,
            "project-a",
            DirectScanPolicy {
                max_file_bytes: 64,
                max_files: 2,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn query_limits_are_explicit() {
        assert!(normalized_query_terms("").is_err());
        assert!(normalized_query_terms(&"x".repeat(DIRECT_SCAN_QUERY_MAX_BYTES + 1)).is_err());
        let too_many = (0..=DIRECT_SCAN_QUERY_MAX_TERMS)
            .map(|index| format!("q{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(normalized_query_terms(&too_many).is_err());
    }
}
