pub fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn format_tokens_comma(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}

pub fn format_cost(usd: f64) -> String {
    if usd >= 1.0 {
        format!("${:.2}", usd)
    } else if usd > 0.0 {
        format!("${:.4}", usd)
    } else {
        "N/A".to_string()
    }
}

pub fn format_remaining(minutes: i64) -> String {
    if minutes <= 0 {
        return "expired".to_string();
    }
    let h = minutes / 60;
    let m = minutes % 60;
    if h > 0 {
        format!("{}h {}m left", h, m)
    } else {
        format!("{}m left", m)
    }
}
