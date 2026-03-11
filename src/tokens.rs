use crate::models::{ContextWindow, CurrentUsageField};

pub struct TokenMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub total_tokens: u64,
}

pub fn compute_token_metrics(ctx: &ContextWindow) -> TokenMetrics {
    let input_tokens = ctx.total_input_tokens.unwrap_or(0);
    let output_tokens = ctx.total_output_tokens.unwrap_or(0);

    let (cached_tokens, current_usage_total) = match &ctx.current_usage {
        Some(CurrentUsageField::Obj(u)) => {
            let cached = u.cache_creation_input_tokens + u.cache_read_input_tokens;
            let cu_input = u.input_tokens.unwrap_or(0);
            let cu_output = u.output_tokens.unwrap_or(0);
            let total = cu_input + cu_output + cached;
            (cached, Some(total))
        }
        Some(CurrentUsageField::Number(n)) => (0, Some(*n as u64)),
        None => (0, None),
    };

    let total_tokens = current_usage_total.unwrap_or(input_tokens + output_tokens);

    TokenMetrics {
        input_tokens,
        output_tokens,
        cached_tokens,
        total_tokens,
    }
}
