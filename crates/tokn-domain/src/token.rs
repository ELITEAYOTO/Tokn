use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    pub cache_write_input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_output_tokens: Option<u64>,
    pub reported_total_tokens: Option<u64>,
}

impl TokenUsage {
    pub fn logical_total(&self) -> Option<u64> {
        Some(self.input_tokens?.saturating_add(self.output_tokens?))
    }

    pub fn ordinary_uncached(&self) -> Option<u64> {
        let input = self.input_tokens?;
        let cached = self.cached_input_tokens?;
        let after_cached = input.checked_sub(cached)?;
        match self.cache_write_input_tokens {
            Some(writes) => after_cached.checked_sub(writes),
            None => Some(after_cached),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenTotals {
    pub usage_records: u64,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub cache_write_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub input_known: u64,
    pub cached_input_known: u64,
    pub cache_write_input_known: u64,
    pub output_known: u64,
    pub reasoning_output_known: u64,
}

impl TokenTotals {
    pub fn add_usage(&mut self, usage: &TokenUsage) {
        self.usage_records += 1;
        if let Some(value) = usage.input_tokens {
            self.input_tokens = self.input_tokens.saturating_add(value);
            self.input_known += 1;
        }
        if let Some(value) = usage.cached_input_tokens {
            self.cached_input_tokens = self.cached_input_tokens.saturating_add(value);
            self.cached_input_known += 1;
        }
        if let Some(value) = usage.cache_write_input_tokens {
            self.cache_write_input_tokens = self.cache_write_input_tokens.saturating_add(value);
            self.cache_write_input_known += 1;
        }
        if let Some(value) = usage.output_tokens {
            self.output_tokens = self.output_tokens.saturating_add(value);
            self.output_known += 1;
        }
        if let Some(value) = usage.reasoning_output_tokens {
            self.reasoning_output_tokens = self.reasoning_output_tokens.saturating_add(value);
            self.reasoning_output_known += 1;
        }
    }

    pub fn add_totals(&mut self, other: &Self) {
        self.usage_records = self.usage_records.saturating_add(other.usage_records);
        self.input_tokens = self.input_tokens.saturating_add(other.input_tokens);
        self.cached_input_tokens = self
            .cached_input_tokens
            .saturating_add(other.cached_input_tokens);
        self.cache_write_input_tokens = self
            .cache_write_input_tokens
            .saturating_add(other.cache_write_input_tokens);
        self.output_tokens = self.output_tokens.saturating_add(other.output_tokens);
        self.reasoning_output_tokens = self
            .reasoning_output_tokens
            .saturating_add(other.reasoning_output_tokens);
        self.input_known = self.input_known.saturating_add(other.input_known);
        self.cached_input_known = self
            .cached_input_known
            .saturating_add(other.cached_input_known);
        self.cache_write_input_known = self
            .cache_write_input_known
            .saturating_add(other.cache_write_input_known);
        self.output_known = self.output_known.saturating_add(other.output_known);
        self.reasoning_output_known = self
            .reasoning_output_known
            .saturating_add(other.reasoning_output_known);
    }

    pub fn logical_total(&self) -> Option<u64> {
        if self.input_known == self.usage_records && self.output_known == self.usage_records {
            Some(self.input_tokens.saturating_add(self.output_tokens))
        } else {
            None
        }
    }

    pub fn ordinary_uncached_input(&self) -> Option<u64> {
        if self.input_known != self.usage_records || self.cached_input_known != self.usage_records {
            return None;
        }
        self.input_tokens.checked_sub(self.cached_input_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uncached_requires_cached_input_but_not_cache_write() {
        let usage = TokenUsage {
            input_tokens: Some(100),
            cached_input_tokens: Some(70),
            cache_write_input_tokens: None,
            ..Default::default()
        };
        assert_eq!(usage.ordinary_uncached(), Some(30));

        let unknown_cached = TokenUsage {
            input_tokens: Some(100),
            cached_input_tokens: None,
            ..Default::default()
        };
        assert_eq!(unknown_cached.ordinary_uncached(), None);
    }
}
