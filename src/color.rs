pub const BRIGHT_BLUE: &str = "\x1b[38;5;111m";
pub const RST: &str = "\x1b[0m";

pub fn c(text: &str, color: &str) -> String {
    format!("{}{}{}", color, text, RST)
}
