use tokn_domain::TokenUsage;

#[derive(Debug, Default, Clone)]
pub struct UsageSummary {
    pub records: u64,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub cache_write_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
}

impl UsageSummary {
    pub fn add(&mut self, usage: &TokenUsage) {
        self.records += 1;
        self.input_tokens += usage.input_tokens.unwrap_or(0);
        self.cached_input_tokens += usage.cached_input_tokens.unwrap_or(0);
        self.cache_write_input_tokens += usage.cache_write_input_tokens.unwrap_or(0);
        self.output_tokens += usage.output_tokens.unwrap_or(0);
        self.reasoning_output_tokens += usage.reasoning_output_tokens.unwrap_or(0);
    }

    pub fn logical_total(&self) -> u64 {
        self.input_tokens.saturating_add(self.output_tokens)
    }
}
