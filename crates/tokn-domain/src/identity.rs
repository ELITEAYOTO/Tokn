
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
}
