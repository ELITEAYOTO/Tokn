use tokn_domain::TokenUsage;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenInvariantViolation {
    CachedExceedsInput,
    ReasoningExceedsOutput,
    CacheWriteExceedsInput,
}

pub fn validate_usage(usage: &TokenUsage) -> Vec<TokenInvariantViolation> {
    let mut issues = Vec::new();

    if let (Some(input), Some(cached)) = (usage.input_tokens, usage.cached_input_tokens)
        && cached > input
    {
        issues.push(TokenInvariantViolation::CachedExceedsInput);
    }

    if let (Some(output), Some(reasoning)) = (usage.output_tokens, usage.reasoning_output_tokens)
        && reasoning > output
    {
        issues.push(TokenInvariantViolation::ReasoningExceedsOutput);
    }

    if let (Some(input), Some(writes)) = (usage.input_tokens, usage.cache_write_input_tokens)
        && writes > input
    {
        issues.push(TokenInvariantViolation::CacheWriteExceedsInput);
    }

    issues
}
