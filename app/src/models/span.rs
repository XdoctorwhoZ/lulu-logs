use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanPhase {
    Beg,
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpanEvent {
    pub phase: SpanPhase,
    pub span_id: String,
    pub name: Option<String>,
    pub kind: String,
    pub success: Option<bool>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub metadata: Option<Value>,
    pub result: Option<Value>,
}

impl SpanEvent {
    pub fn is_scenario(&self) -> bool {
        self.kind == "scenario"
    }

    pub fn is_step(&self) -> bool {
        self.kind == "step"
    }
}

pub fn is_span_type(type_str: &str) -> bool {
    matches!(
        type_str,
        "span_beg"
            | "span_end"
            | "scenario_beg"
            | "scenario_end"
            | "tool_call_beg"
            | "tool_call_end"
            | "step_beg"
            | "step_end"
    )
}

pub fn is_span_begin_type(type_str: &str) -> bool {
    matches!(type_str, "span_beg" | "scenario_beg" | "tool_call_beg" | "step_beg")
}

fn implied_kind(type_str: &str) -> Option<&'static str> {
    match type_str {
        "scenario_beg" | "scenario_end" => Some("scenario"),
        "tool_call_beg" | "tool_call_end" => Some("tool_call"),
        "step_beg" | "step_end" => Some("step"),
        _ => None,
    }
}

pub fn parse_span_event(type_str: &str, data: &[u8]) -> Option<SpanEvent> {
    if !is_span_type(type_str) {
        return None;
    }

    let value = serde_json::from_slice::<Value>(data).ok()?;
    let phase = if is_span_begin_type(type_str) {
        SpanPhase::Beg
    } else {
        SpanPhase::End
    };
    let span_id = value.get("span_id")?.as_str()?.to_string();
    let kind = implied_kind(type_str)
        .map(str::to_string)
        .or_else(|| value.get("kind").and_then(|v| v.as_str()).map(str::to_string))
        .unwrap_or_else(|| "span".to_string());

    Some(SpanEvent {
        phase,
        span_id,
        name: value.get("name").and_then(|v| v.as_str()).map(str::to_string),
        kind,
        success: value.get("success").and_then(|v| v.as_bool()),
        error: value.get("error").and_then(|v| v.as_str()).map(str::to_string),
        duration_ms: value.get("duration_ms").and_then(|v| v.as_u64()),
        metadata: value.get("metadata").cloned(),
        result: value.get("result").cloned(),
    })
}
