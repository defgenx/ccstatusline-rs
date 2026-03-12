mod color;
mod format;
mod git;
mod models;
mod tokens;

use color::{c, BRIGHT_BLUE, RST};
use format::format_tokens;
use git::git_branch;
use models::Input;
use std::io::BufRead;
use std::process::{Command, Stdio};
use tokens::compute_token_metrics;

/// Print a line with ANSI reset prefix and non-breaking spaces to prevent trimming.
fn emit(line: &str) {
    let output = line.replace(' ', "\u{00A0}");
    println!("\x1b[0m{}", output);
}

fn main() {
    let mut input_str = String::new();
    let _ = std::io::stdin().lock().read_line(&mut input_str);

    let input: Input = serde_json::from_str(&input_str).unwrap_or_default();
    let cwd = input.resolved_cwd().unwrap_or("?").to_string();

    // Line 1: custom-command widget — run ccusage, pass status JSON as stdin
    if let Some(ccusage_output) = run_custom_command("bunx -y ccusage@latest statusline", &input_str) {
        emit(&ccusage_output);
    }

    // Line 2: current-working-dir | separator | git-branch
    let branch_str = match git_branch(&cwd) {
        Some(b) => format!("\u{2387} {}", b),
        None => "\u{2387} no git".to_string(),
    };
    emit(&format!(
        "{}{} | {}",
        RST,
        c(&format!("cwd: {}", cwd), BRIGHT_BLUE),
        c(&branch_str, BRIGHT_BLUE),
    ));

    // Line 3: tokens-total | tokens-input | tokens-output | tokens-cached
    let ctx = input.context_window.unwrap_or_default();
    let metrics = compute_token_metrics(&ctx, input.transcript_path.as_deref());
    emit(&format!(
        "{}{} | {} | {} | {}",
        RST,
        c(&format!("Total: {}", format_tokens(metrics.total_tokens)), BRIGHT_BLUE),
        c(&format!("In: {}", format_tokens(metrics.input_tokens)), BRIGHT_BLUE),
        c(&format!("Out: {}", format_tokens(metrics.output_tokens)), BRIGHT_BLUE),
        c(&format!("Cached: {}", format_tokens(metrics.cached_tokens)), BRIGHT_BLUE),
    ));
}

/// Mirrors CustomCommand widget: spawn command, pipe status JSON as stdin, return stdout.
fn run_custom_command(command: &str, stdin_data: &str) -> Option<String> {
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = parts.split_first()?;

    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;

    if let Some(mut child_stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = child_stdin.write_all(stdin_data.as_bytes());
    }

    let output = child.wait_with_output().ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}
