use crate::models::ContextWindow;
use serde_json::Value;
use std::fs;
use std::io::BufRead;

pub struct TokenMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub total_tokens: u64,
}

struct TranscriptTokens {
    input_tokens: u64,
    output_tokens: u64,
    cached_tokens: u64,
}

/// Matches the reference ccstatusline widget behavior:
/// - tokens-input:  context_window.total_input_tokens, fallback transcript
/// - tokens-output: context_window.total_output_tokens, fallback transcript
/// - tokens-cached: transcript only
/// - tokens-total:  transcript only (input + output + cached)
pub fn compute_token_metrics(ctx: &ContextWindow, transcript_path: Option<&str>) -> TokenMetrics {
    let transcript = transcript_path.and_then(read_transcript_tokens);

    // Input: status JSON first, fallback transcript
    let input_tokens = ctx
        .total_input_tokens
        .or(transcript.as_ref().map(|t| t.input_tokens))
        .unwrap_or(0);

    // Output: status JSON first, fallback transcript
    let output_tokens = ctx
        .total_output_tokens
        .or(transcript.as_ref().map(|t| t.output_tokens))
        .unwrap_or(0);

    // Cached: transcript only
    let cached_tokens = transcript.as_ref().map(|t| t.cached_tokens).unwrap_or(0);

    // Total: transcript only (input + output + cached from transcript)
    let total_tokens = match &transcript {
        Some(t) => t.input_tokens + t.output_tokens + t.cached_tokens,
        None => input_tokens + output_tokens,
    };

    TokenMetrics {
        input_tokens,
        output_tokens,
        cached_tokens,
        total_tokens,
    }
}

/// Read transcript JSONL, sum all message.usage entries.
/// Mirrors getTokenMetrics() from jsonl-metrics.ts.
fn read_transcript_tokens(path: &str) -> Option<TranscriptTokens> {
    let file = fs::File::open(path).ok()?;
    let reader = std::io::BufReader::new(file);

    let mut input = 0u64;
    let mut output = 0u64;
    let mut cached = 0u64;

    for line in reader.lines().flatten() {
        let record: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let usage = match record.get("message").and_then(|m| m.get("usage")) {
            Some(u) => u,
            None => continue,
        };

        input += usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        output += usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        cached += usage.get("cache_read_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
        cached += usage.get("cache_creation_input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    }

    Some(TranscriptTokens {
        input_tokens: input,
        output_tokens: output,
        cached_tokens: cached,
    })
}
