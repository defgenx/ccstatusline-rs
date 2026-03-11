use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Default)]
pub struct Input {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub context_window: Option<ContextWindow>,
    #[serde(default)]
    pub model: Option<ModelField>,
    #[serde(default)]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ModelField {
    Str(String),
    Obj(ModelObj),
}

#[derive(Deserialize)]
pub struct ModelObj {
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Deserialize, Default)]
pub struct ContextWindow {
    #[serde(default, deserialize_with = "deser_opt_u64")]
    pub total_input_tokens: Option<u64>,
    #[serde(default, deserialize_with = "deser_opt_u64")]
    pub total_output_tokens: Option<u64>,
    #[serde(default)]
    pub current_usage: Option<CurrentUsageField>,
    #[serde(default, deserialize_with = "deser_opt_u64")]
    pub context_window_size: Option<u64>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum CurrentUsageField {
    Number(f64),
    Obj(CurrentUsageObj),
}

#[derive(Deserialize, Default)]
pub struct CurrentUsageObj {
    #[serde(default)]
    pub input_tokens: Option<u64>,
    #[serde(default)]
    pub output_tokens: Option<u64>,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

#[derive(Deserialize, Default)]
pub struct Cost {
    #[serde(default, deserialize_with = "deser_opt_f64")]
    pub total_cost_usd: Option<f64>,
}

pub fn deser_opt_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
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

pub fn deser_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v: Option<Value> = Option::deserialize(deserializer)?;
    match v {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => Ok(n.as_f64()),
        Some(Value::String(s)) => Ok(s.parse().ok()),
        _ => Ok(None),
    }
}

pub fn model_display_name(input: &Input) -> String {
    match &input.model {
        Some(ModelField::Obj(obj)) => obj
            .display_name
            .clone()
            .or_else(|| obj.id.clone())
            .unwrap_or_else(|| "?".to_string()),
        Some(ModelField::Str(s)) => s.clone(),
        None => "?".to_string(),
    }
}
