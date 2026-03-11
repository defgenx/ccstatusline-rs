pub const BRIGHT_BLUE: &str = "\x1b[38;5;111m";
#[allow(dead_code)]
pub const CYAN: &str = "\x1b[38;5;81m";
pub const YELLOW: &str = "\x1b[38;5;220m";
pub const GREEN: &str = "\x1b[38;5;114m";
pub const RED: &str = "\x1b[38;5;196m";
#[allow(dead_code)]
pub const ORANGE: &str = "\x1b[38;5;208m";
#[allow(dead_code)]
pub const MAGENTA: &str = "\x1b[38;5;176m";
pub const RST: &str = "\x1b[0m";

pub fn c(text: &str, color: &str) -> String {
    format!("{}{}{}", color, text, RST)
}

pub fn context_pct_colored(pct: u64) -> String {
    let s = format!("{}%", pct);
    if pct < 50 {
        c(&s, GREEN)
    } else if pct < 80 {
        c(&s, YELLOW)
    } else {
        c(&s, RED)
    }
}
