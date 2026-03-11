mod color;
mod cost;
mod format;
mod git;
mod models;
mod tokens;

use color::{c, context_pct_colored, BRIGHT_BLUE, RST};
use cost::compute_daily_cost;
use format::{format_cost, format_remaining, format_tokens, format_tokens_comma};
use git::git_branch;
use models::{model_display_name, Input};
use std::io::BufRead;
use tokens::compute_token_metrics;

fn main() {
    let mut input_str = String::new();
    let _ = std::io::stdin().lock().read_line(&mut input_str);

    let input: Input = serde_json::from_str(&input_str).unwrap_or_default();

    let cwd = input.cwd.as_deref().unwrap_or("?").to_string();
    let model_name = model_display_name(&input);
    let session_cost_str = input
        .cost
        .as_ref()
        .and_then(|c| c.total_cost_usd)
        .map(format_cost)
        .unwrap_or_else(|| "N/A".to_string());

    let ctx = input.context_window.unwrap_or_default();
    let metrics = compute_token_metrics(&ctx);
    let ctx_size = ctx.context_window_size.unwrap_or(0);

    let daily = compute_daily_cost();

    // Line 1: Model | Costs | Burn rate | Context usage
    let mut cost_parts = vec![format!("{} session", session_cost_str)];
    if let Some(ref d) = daily {
        if d.today_cost > 0.0 {
            cost_parts.push(format!("{} today", format_cost(d.today_cost)));
        }
        if d.block_cost > 0.0 {
            cost_parts.push(format!(
                "{} block ({})",
                format_cost(d.block_cost),
                format_remaining(d.block_remaining_min)
            ));
        }
    }

    let burn_part = match &daily {
        Some(d) if d.burn_rate_per_hour > 0.01 => {
            format!(" | \u{1F525} {}/hr", format_cost(d.burn_rate_per_hour))
        }
        _ => String::new(),
    };

    let ctx_part = if ctx_size > 0 {
        let pct = (metrics.total_tokens as f64 / ctx_size as f64 * 100.0) as u64;
        format!(
            " | \u{1F9E0} {} ({})",
            format_tokens_comma(metrics.total_tokens),
            context_pct_colored(pct),
        )
    } else {
        String::new()
    };

    println!(
        "{}\u{1F916} {} | \u{1F4B0} {}{}{}",
        RST,
        model_name,
        cost_parts.join(" / "),
        burn_part,
        ctx_part,
    );

    // Line 2: cwd | git branch
    let branch_str = match git_branch(&cwd) {
        Some(b) => format!("\u{2387} {}", b),
        None => "\u{2387} no git".to_string(),
    };
    println!(
        "{}{} | {}",
        RST,
        c(&format!("cwd: {}", cwd), BRIGHT_BLUE),
        c(&branch_str, BRIGHT_BLUE),
    );

    // Line 3: Token breakdown
    println!(
        "{}{} | {} | {} | {}",
        RST,
        c(
            &format!("Total: {}", format_tokens(metrics.total_tokens)),
            BRIGHT_BLUE
        ),
        c(
            &format!("In: {}", format_tokens(metrics.input_tokens)),
            BRIGHT_BLUE
        ),
        c(
            &format!("Out: {}", format_tokens(metrics.output_tokens)),
            BRIGHT_BLUE
        ),
        c(
            &format!("Cached: {}", format_tokens(metrics.cached_tokens)),
            BRIGHT_BLUE
        ),
    );
}
