# ccstatusline

Rust replacement for the npm `ccstatusline` package. A fast, native status line for Claude Code CLI.

## What it does

Reads Claude Code's JSON status data from stdin and outputs ANSI-colored status lines:
- Line 1: current working directory + git branch
- Line 2: token stats (total, input, output, cached)

## Claude Code status line protocol

Claude Code pipes JSON to stdin with these fields:
- `workspace.current_dir` / `cwd` — current working directory
- `context_window.total_input_tokens` / `total_output_tokens` — cumulative token counts
- `context_window.current_usage.cache_read_input_tokens` / `cache_creation_input_tokens` — cache stats

The command outputs multi-line ANSI-colored text to stdout.

## Build & install

```sh
cargo build --release
cp target/release/ccstatusline ~/.claude/ccstatusline-bin
```

## Configuration

In `~/.claude/settings.json`:
```json
{
  "statusLine": {
    "type": "command",
    "command": "/Users/adelvecchio/.claude/ccstatusline-bin",
    "padding": 0
  }
}
```

## Style

- Color: ANSI 256-color blue (`38;5;111`)
- Tokens formatted as raw number, `Xk`, or `X.XM`
- CWD truncated to 40 chars
- Git branch via `git rev-parse --abbrev-ref HEAD`

## Dependencies

Only `serde` and `serde_json`. No runtime dependencies.
