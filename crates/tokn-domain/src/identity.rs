use serde::{Deserialize, Serialize};

pub const CONTEXT_IDENTITY_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvidenceIdentityCoverage {
    Observed,
    Partial,
    #[default]
    NotCaptured,
    Unknown,
}

impl EvidenceIdentityCoverage {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Observed => "OBSERVED",
            Self::Partial => "PARTIAL",
            Self::NotCaptured => "NOT_CAPTURED",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "OBSERVED" => Some(Self::Observed),
            "PARTIAL" => Some(Self::Partial),
            "NOT_CAPTURED" => Some(Self::NotCaptured),
            "UNKNOWN" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Derive the accepted project-scoped stable logical source identity.
///
/// This implementation is shared by Store ingestion and future shadow/index
/// mechanics. Changing any derivation/domain bytes is a compatibility break for
/// existing `src-v1-*` identities.
pub fn scoped_source_id_bytes(scope_key: &str, locator: &[u8]) -> String {
    let key = blake3::derive_key(
        "tokn.project-scoped-source-identity.v1",
        scope_key.as_bytes(),
    );
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"logical-source-locator-v1");
    hasher.update(&[0]);
    hasher.update(locator);
    format!("src-v1-{}", hasher.finalize().to_hex())
}

/// Derive a project-scoped exact whole-file identity for shadow-index
/// correctness/invalidation.
///
/// `ixc-v1-*` is intentionally domain-separated from SourceStableId,
/// ContentFingerprint and SourceVersionFingerprint. Equality across those
/// identity domains must never be inferred from digest text.
pub fn scoped_shadow_document_id_bytes(
    scope_key: &str,
    source_stable_id: &str,
    index_content_hash: &str,
) -> String {
    let key = blake3::derive_key(
        "tokn.project-scoped-shadow-document-id.v1",
        scope_key.as_bytes(),
    );
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"shadow-document-identity-v1");
    hasher.update(&[0]);
    hasher.update(source_stable_id.as_bytes());
    hasher.update(&[0]);
    hasher.update(index_content_hash.as_bytes());
    format!("sdoc-v1-{}", hasher.finalize().to_hex())
}

pub fn scoped_index_content_hash_bytes(scope_key: &str, bytes: &[u8]) -> String {
    let key = blake3::derive_key(
        "tokn.project-scoped-index-content-hash.v1",
        scope_key.as_bytes(),
    );
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"repository-file-bytes-v1");
    hasher.update(&[0]);
    hasher.update(bytes);
    format!("ixc-v1-{}", hasher.finalize().to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_coverage_round_trips_known_values() {
        for value in [
            EvidenceIdentityCoverage::Observed,
            EvidenceIdentityCoverage::Partial,
            EvidenceIdentityCoverage::NotCaptured,
            EvidenceIdentityCoverage::Unknown,
        ] {
            assert_eq!(EvidenceIdentityCoverage::parse(value.as_str()), Some(value));
        }
        assert_eq!(EvidenceIdentityCoverage::parse("MISSING"), None);
    }

    #[test]
    fn source_stable_id_preserves_the_accepted_v1_golden_vector() {
        assert_eq!(
            scoped_source_id_bytes("project-a", b"file:src/lib.rs"),
            "src-v1-b97958930893f1957c17f4d486a7e2e2472ff9b3dfe0421a29e2061bbd2e3257"
        );
    }

    #[test]
    fn source_stable_id_is_project_scoped_and_locator_sensitive() {
        let baseline = scoped_source_id_bytes("project-a", b"file:src/lib.rs");
        let repeat = scoped_source_id_bytes("project-a", b"file:src/lib.rs");
        let other_project = scoped_source_id_bytes("project-b", b"file:src/lib.rs");
        let other_locator = scoped_source_id_bytes("project-a", b"file:src/main.rs");

        assert_eq!(baseline, repeat);
        assert_ne!(baseline, other_project);
        assert_ne!(baseline, other_locator);
    }

    #[test]
    fn shadow_document_id_is_deterministic_project_scoped_and_domain_separated() {
        let source = scoped_source_id_bytes("project-a", b"file:src/lib.rs");
        let content = scoped_index_content_hash_bytes("project-a", b"fn alpha() {}\n");
        let baseline = scoped_shadow_document_id_bytes("project-a", &source, &content);
        let repeat = scoped_shadow_document_id_bytes("project-a", &source, &content);
        let other_project = scoped_shadow_document_id_bytes("project-b", &source, &content);
        let other_content = scoped_shadow_document_id_bytes(
            "project-a",
            &source,
            &scoped_index_content_hash_bytes("project-a", b"fn beta() {}\n"),
        );

        assert_eq!(baseline, repeat);
        assert!(baseline.starts_with("sdoc-v1-"));
        assert_ne!(baseline, other_project);
        assert_ne!(baseline, other_content);
        assert_ne!(baseline, source);
        assert_ne!(baseline, content);
    }

    #[test]
    fn index_content_hash_is_project_scoped_content_sensitive_and_domain_separated() {
        let bytes = b"fn main() {}\n";
        let baseline = scoped_index_content_hash_bytes("project-a", bytes);
        let repeat = scoped_index_content_hash_bytes("project-a", bytes);
        let other_project = scoped_index_content_hash_bytes("project-b", bytes);
        let other_content =
            scoped_index_content_hash_bytes("project-a", b"fn main() { panic!() }\n");
        let source_domain = scoped_source_id_bytes("project-a", bytes);

        assert_eq!(baseline, repeat);
        assert!(baseline.starts_with("ixc-v1-"));
        assert_ne!(baseline, other_project);
        assert_ne!(baseline, other_content);
        assert_ne!(baseline, source_domain);
    }
}
