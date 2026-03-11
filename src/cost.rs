use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::io::BufRead;
use std::process::Command;
use std::time::SystemTime;

pub struct DailyCost {
    pub today_cost: f64,
    pub block_cost: f64,
    pub block_remaining_min: i64,
    pub burn_rate_per_hour: f64,
}

fn get_model_pricing(model_id: &str) -> (f64, f64, f64, f64) {
    // (input, output, cache_creation, cache_read) per token
    // Note: cache_creation tokens in Claude Code are all ephemeral (5m/1h),
    // so we set cache_creation cost to 0 to match ccusage behavior.
    if model_id.contains("opus-4-6") || model_id.contains("opus-4-5") {
        (5e-6, 25e-6, 0.0, 5e-7)
    } else if model_id.contains("sonnet") {
        (3e-6, 15e-6, 0.0, 3e-7)
    } else if model_id.contains("haiku") {
        (1e-6, 5e-6, 0.0, 1e-7)
    } else if model_id.contains("opus") {
        (15e-6, 75e-6, 0.0, 1.5e-6)
    } else {
        (3e-6, 15e-6, 0.0, 3e-7)
    }
}

pub fn compute_daily_cost() -> Option<DailyCost> {
    let home = std::env::var("HOME").ok()?;
    let projects_dir = format!("{}/.claude/projects", home);

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();

    let today_prefix = {
        let output = Command::new("date").arg("+%Y-%m-%d").output().ok()?;
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    };

    let mut today_cost = 0.0f64;
    let mut block_start_ts: Option<u64> = None;
    let mut block_cost = 0.0f64;
    let mut block_last_ts: Option<u64> = None;
    let mut seen_ids: HashSet<String> = HashSet::new();

    const BLOCK_WINDOW: u64 = 5 * 3600;

    let _ = scan_dir_recursive(
        &projects_dir,
        &today_prefix,
        now,
        &mut today_cost,
        &mut block_cost,
        &mut block_start_ts,
        &mut block_last_ts,
        &mut seen_ids,
    );

    let block_remaining = if let Some(start) = block_start_ts {
        let end = start + BLOCK_WINDOW;
        if now < end {
            ((end - now) / 60) as i64
        } else {
            0
        }
    } else {
        0
    };

    let burn_rate = if let (Some(start), Some(last)) = (block_start_ts, block_last_ts) {
        let elapsed_hours = (last - start) as f64 / 3600.0;
        if elapsed_hours > 0.01 {
            block_cost / elapsed_hours
        } else {
            0.0
        }
    } else {
        0.0
    };

    Some(DailyCost {
        today_cost,
        block_cost,
        block_remaining_min: block_remaining,
        burn_rate_per_hour: burn_rate,
    })
}

fn scan_dir_recursive(
    dir: &str,
    today_prefix: &str,
    now: u64,
    today_cost: &mut f64,
    block_cost: &mut f64,
    block_start_ts: &mut Option<u64>,
    block_last_ts: &mut Option<u64>,
    seen_ids: &mut HashSet<String>,
) -> Option<()> {
    const GAP_THRESHOLD: u64 = 30 * 60;

    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir_recursive(
                path.to_str()?,
                today_prefix,
                now,
                today_cost,
                block_cost,
                block_start_ts,
                block_last_ts,
                seen_ids,
            );
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("jsonl") {
            continue;
        }
        // Skip files not modified in the last 24h
        if let Ok(meta) = fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                let mod_secs = modified
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now - mod_secs > 86400 {
                    continue;
                }
            }
        }

        let file = fs::File::open(&path).ok()?;
        let reader = std::io::BufReader::new(file);
        for line in reader.lines().flatten() {
            if !line.contains("\"usage\"") || !line.contains("\"stop_reason\"") {
                continue;
            }
            let record: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let stop_reason = record.get("message").and_then(|m| m.get("stop_reason"));
            if !matches!(stop_reason, Some(Value::String(_))) {
                continue;
            }

            let timestamp_str = record
                .get("timestamp")
                .or_else(|| record.get("message").and_then(|m| m.get("timestamp")))
                .and_then(|v| v.as_str());

            if !timestamp_str
                .map(|ts| ts.starts_with(today_prefix))
                .unwrap_or(false)
            {
                continue;
            }

            let ts_epoch = timestamp_str.and_then(parse_iso_epoch);

            let msg_id = record
                .get("message")
                .and_then(|m| m.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !msg_id.is_empty() && !seen_ids.insert(msg_id.to_string()) {
                continue;
            }

            if let Some(usage) = record.get("message").and_then(|m| m.get("usage")) {
                let model = record
                    .get("message")
                    .and_then(|m| m.get("model"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let input = usage.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let output = usage.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
                let cache_create = usage
                    .get("cache_creation_input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let cache_read = usage
                    .get("cache_read_input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                let (p_in, p_out, p_cc, p_cr) = get_model_pricing(model);
                let cost = input as f64 * p_in
                    + output as f64 * p_out
                    + cache_create as f64 * p_cc
                    + cache_read as f64 * p_cr;

                *today_cost += cost;

                if let Some(ts) = ts_epoch {
                    if let Some(last) = *block_last_ts {
                        if ts > last + GAP_THRESHOLD {
                            *block_start_ts = Some(ts);
                            *block_cost = cost;
                        } else {
                            *block_cost += cost;
                        }
                    } else {
                        *block_start_ts = Some(ts);
                        *block_cost = cost;
                    }
                    *block_last_ts = Some(ts);
                }
            }
        }
    }
    Some(())
}

fn parse_iso_epoch(ts: &str) -> Option<u64> {
    if ts.len() < 19 {
        return None;
    }
    let year: u64 = ts[0..4].parse().ok()?;
    let month: u64 = ts[5..7].parse().ok()?;
    let day: u64 = ts[8..10].parse().ok()?;
    let hour: u64 = ts[11..13].parse().ok()?;
    let min: u64 = ts[14..16].parse().ok()?;
    let sec: u64 = ts[17..19].parse().ok()?;

    let days_since_epoch = (year - 1970) * 365
        + (year - 1969) / 4
        - (year - 1901) / 100
        + (year - 1601) / 400
        + month_days(month, year)
        + day
        - 1;

    Some(days_since_epoch * 86400 + hour * 3600 + min * 60 + sec)
}

fn month_days(month: u64, year: u64) -> u64 {
    let days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let d = if (1..=12).contains(&month) {
        days[(month - 1) as usize]
    } else {
        0
    };
    if month > 2 && (year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) {
        d + 1
    } else {
        d
    }
}
