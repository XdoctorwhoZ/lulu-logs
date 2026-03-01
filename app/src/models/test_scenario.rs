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

/// A test scenario tracked by correlating `beg_test_scenario` and `end_test_scenario` log entries.
///
/// Scenarios are identified by `(name, source)` — the same scenario name from different
/// MQTT sources are treated as distinct scenarios (per lulu-logs v1.1.0 §3.4).
#[derive(Debug, Clone, PartialEq)]
pub struct TestScenario {
    /// Scenario identifier (from the JSON `name` field).
    pub name: String,

    /// MQTT source that emitted this scenario.
    pub source: String,

    /// MQTT attribute on which beg/end are published.
    pub attribute: String,

    /// Timestamp of the `beg_test_scenario` entry.
    pub beg_timestamp: String,

    /// Timestamp of the `end_test_scenario` entry (if received).
    pub end_timestamp: Option<String>,

    /// Index into `AppState.logs` of the `beg_test_scenario` entry.
    pub beg_log_index: usize,

    /// Index into `AppState.logs` of the `end_test_scenario` entry (if received).
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
