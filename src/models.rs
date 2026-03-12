use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Default)]
pub struct Input {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub workspace: Option<Workspace>,
    #[serde(default)]
    pub context_window: Option<ContextWindow>,
    #[serde(default)]
    pub transcript_path: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct Workspace {
    #[serde(default)]
    pub current_dir: Option<String>,
}

impl Input {
    pub fn resolved_cwd(&self) -> Option<&str> {
        self.cwd
            .as_deref()
            .or_else(|| self.workspace.as_ref().and_then(|w| w.current_dir.as_deref()))
    }
}

#[derive(Deserialize, Default)]
pub struct ContextWindow {
    #[serde(default, deserialize_with = "deser_opt_u64")]
    pub total_input_tokens: Option<u64>,
    #[serde(default, deserialize_with = "deser_opt_u64")]
    pub total_output_tokens: Option<u64>,
}

fn deser_opt_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<Value> = Option::deserialize(deserializer)?;
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_u64().or_else(|| n.as_f64().map(|f| f as u64))),
        Some(Value::String(s)) => Ok(s.parse().ok()),
        _ => Ok(None),
    }
}
