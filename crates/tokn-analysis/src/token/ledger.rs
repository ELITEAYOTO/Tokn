use std::collections::HashMap;

use tokn_domain::TokenUsage;

use crate::{UsageSummary, validate_usage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LedgerInsert {
    Accepted,
    Duplicate,
    Conflict,
    Invalid,
}

#[derive(Debug, Default, Clone)]
pub struct TokenLedger {
    pub accepted: Vec<TokenUsage>,
    pub duplicates_suppressed: u64,
    pub usage_conflicts: u64,
    pub rejected_invariant_count: u64,
    keyed: HashMap<String, TokenUsage>,
}

impl TokenLedger {
    pub fn push(&mut self, usage: TokenUsage) -> LedgerInsert {
        self.push_keyed(None, usage)
    }

    pub fn push_keyed(&mut self, key: Option<String>, usage: TokenUsage) -> LedgerInsert {
        if !validate_usage(&usage).is_empty() {
            self.rejected_invariant_count += 1;
            return LedgerInsert::Invalid;
        }

        if let Some(key) = key {
            if let Some(existing) = self.keyed.get(&key) {
                if existing == &usage {
                    self.duplicates_suppressed += 1;
                    return LedgerInsert::Duplicate;
                }
                self.usage_conflicts += 1;
                return LedgerInsert::Conflict;
            }
            self.keyed.insert(key, usage.clone());
        }

        self.accepted.push(usage);
        LedgerInsert::Accepted
    }

    pub fn summary(&self) -> UsageSummary {
        let mut out = UsageSummary::default();
        for usage in &self.accepted {
            out.add(usage);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usage(input: u64, output: u64) -> TokenUsage {
        TokenUsage {
            input_tokens: Some(input),
            cached_input_tokens: Some(input / 2),
            output_tokens: Some(output),
            reasoning_output_tokens: Some(output / 2),
            ..Default::default()
        }
    }

    #[test]
    fn logical_total_is_input_plus_output_only() {
        let mut ledger = TokenLedger::default();
        ledger.push(usage(1_000, 200));
        assert_eq!(ledger.summary().logical_total(), 1_200);
    }

    #[test]
    fn exact_duplicate_is_suppressed() {
        let mut ledger = TokenLedger::default();
        assert_eq!(
            ledger.push_keyed(Some("resp:1".into()), usage(100, 20)),
            LedgerInsert::Accepted
        );
        assert_eq!(
            ledger.push_keyed(Some("resp:1".into()), usage(100, 20)),
            LedgerInsert::Duplicate
        );
        assert_eq!(ledger.accepted.len(), 1);
        assert_eq!(ledger.duplicates_suppressed, 1);
    }

    #[test]
    fn conflicting_duplicate_is_preserved_as_conflict_not_double_counted() {
        let mut ledger = TokenLedger::default();
        ledger.push_keyed(Some("resp:1".into()), usage(100, 20));
        assert_eq!(
            ledger.push_keyed(Some("resp:1".into()), usage(120, 20)),
            LedgerInsert::Conflict
        );
        assert_eq!(ledger.accepted.len(), 1);
        assert_eq!(ledger.usage_conflicts, 1);
    }

    #[test]
    fn invariant_violation_is_not_silently_repaired() {
        let mut ledger = TokenLedger::default();
        let bad = TokenUsage {
            input_tokens: Some(10),
            cached_input_tokens: Some(11),
            ..Default::default()
        };
        assert_eq!(ledger.push(bad), LedgerInsert::Invalid);
        assert_eq!(ledger.rejected_invariant_count, 1);
        assert!(ledger.accepted.is_empty());
    }
}
