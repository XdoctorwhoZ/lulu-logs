/// Status of a test scenario lifecycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioStatus {
    /// Scenario has started but not yet ended.
    InProgress,
    /// Scenario ended with success.
    Success,
    /// Scenario ended with failure (contains the error message).
    Failure(String),
}

/// A test scenario tracked by correlating span lifecycle log entries.
///
/// Scenarios are identified by `(span_id, source)`.
#[derive(Debug, Clone, PartialEq)]
pub struct TestScenario {
    /// Stable correlation identifier shared by the begin and end events.
    pub span_id: String,

    /// Scenario identifier (from the JSON `name` field).
    pub name: String,

    /// MQTT source that emitted this scenario.
    pub source: String,

    /// MQTT attribute on which beg/end are published.
    pub attribute: String,

    /// Timestamp of the scenario begin entry.
    pub beg_timestamp: String,

    /// Timestamp of the scenario end entry (if received).
    pub end_timestamp: Option<String>,

    /// Index into `AppState.logs` of the scenario begin entry.
    pub beg_log_index: usize,

    /// Index into `AppState.logs` of the scenario end entry (if received).
    pub end_log_index: Option<usize>,

    /// Current status.
    pub status: ScenarioStatus,
}

impl TestScenario {
    /// Returns the CSS class suffix for this scenario's status badge.
    pub fn status_css_class(&self) -> &'static str {
        match &self.status {
            ScenarioStatus::InProgress => "scenario-badge-pending",
            ScenarioStatus::Success => "scenario-badge-success",
            ScenarioStatus::Failure(_) => "scenario-badge-fail",
        }
    }

    /// Returns a short status label.
    pub fn status_label(&self) -> &'static str {
        match &self.status {
            ScenarioStatus::InProgress => "⏳ Running",
            ScenarioStatus::Success => "✅ Pass",
            ScenarioStatus::Failure(_) => "❌ Fail",
        }
    }

    /// Returns the error message if the scenario failed, `None` otherwise.
    pub fn error_message(&self) -> Option<&str> {
        match &self.status {
            ScenarioStatus::Failure(msg) => Some(msg),
            _ => None,
        }
    }

    /// Returns `true` if a log entry at the given index falls within this scenario's bounds.
    pub fn contains_log_index(&self, idx: usize) -> bool {
        let start = self.beg_log_index;
        let end = self.end_log_index.unwrap_or(usize::MAX);
        idx >= start && idx <= end
    }
}
