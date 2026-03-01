use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::LuluLogEntry;
use crate::models::test_scenario::ScenarioStatus;

/// Renders a single log entry row, with special rendering for scenario beg/end entries.
#[component]
pub fn LogItem(entry: LuluLogEntry, log_index: usize) -> Element {
    let mut expanded = use_signal(|| false);
    let state = use_context::<AppState>();
    let level_class = entry.level.css_class();
    let is_json = entry.data_type == "json";
    let is_beg = entry.data_type == "beg_test_scenario";
    let is_end = entry.data_type == "end_test_scenario";
    let is_expandable = is_json || is_beg || is_end;

    // Extract HH:MM:SS.mmm from ISO 8601 timestamp
    let time_display = extract_time(&entry.timestamp);

    // Determine scenario-specific CSS class
    let scenario_class = if is_beg {
        " scenario-beg"
    } else if is_end {
        // Check if success or failure
        let scenarios = state.scenarios.read();
        let is_success = scenarios.iter().any(|s| {
            s.end_log_index == Some(log_index) && matches!(s.status, ScenarioStatus::Success)
        });
        if is_success {
            " scenario-end-success"
        } else {
            " scenario-end-fail"
        }
    } else {
        // Check if this log is inside an active scenario
        let scenarios = state.scenarios.read();
        let in_scenario = scenarios.iter().any(|s| s.contains_log_index(log_index)
            && s.beg_log_index != log_index
            && s.end_log_index != Some(log_index));
        if in_scenario { " in-scenario" } else { "" }
    };

    // Extract scenario name for beg/end display
    let scenario_name = if is_beg || is_end {
        extract_scenario_name(&entry.decoded_value)
    } else {
        None
    };

    rsx! {
        div {
            class: "log-item {level_class}{scenario_class}",
            onclick: move |_| {
                if is_expandable {
                    let current = *expanded.read();
                    expanded.set(!current);
                }
            },

            span { class: "log-timestamp", "{time_display}" }
            span { class: "log-source", "{entry.source}" }
            span { class: "log-attribute", "{entry.attribute}" }
            span { class: "log-level {level_class}", "{entry.level}" }

            // Scenario tag or data type
            if is_beg {
                span { class: "scenario-tag scenario-tag-beg", "BEGIN" }
            } else if is_end {
                {
                    let scenarios = state.scenarios.read();
                    let is_success = scenarios.iter().any(|s| {
                        s.end_log_index == Some(log_index) && matches!(s.status, ScenarioStatus::Success)
                    });
                    if is_success {
                        rsx! { span { class: "scenario-tag scenario-tag-end-success", "END ✅" } }
                    } else {
                        rsx! { span { class: "scenario-tag scenario-tag-end-fail", "END ❌" } }
                    }
                }
            } else {
                span { class: "log-data-type", "{entry.data_type}" }
            }

            // Value display
            if let Some(ref name) = scenario_name {
                span { class: "log-value scenario-name-display", "{name}" }
            } else {
                span { class: "log-value", "{entry.decoded_value}" }
            }
        }
        if *expanded.read() && is_expandable {
            pre { class: "log-value-expanded",
                "{entry.decoded_value}"
            }
        }
    }
}

/// Extracts the scenario name from the decoded JSON value.
fn extract_scenario_name(decoded_value: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(decoded_value)
        .ok()
        .and_then(|v| v.get("name").and_then(|n| n.as_str()).map(String::from))
}

/// Extracts `HH:MM:SS.mmm` from an ISO 8601 timestamp string.
///
/// If the timestamp cannot be parsed, returns the original string truncated to 12 chars.
fn extract_time(timestamp: &str) -> String {
    // ISO 8601 format: "2026-02-26T14:30:00.123Z"
    //                   0123456789012345678901234
    if let Some(t_pos) = timestamp.find('T') {
        let time_part = &timestamp[t_pos + 1..];
        // Take up to 12 chars: "14:30:00.123"
        let end = time_part
            .find('Z')
            .or_else(|| time_part.find('+'))
            .or_else(|| time_part.find('-'))
            .unwrap_or(time_part.len())
            .min(12);
        time_part[..end].to_string()
    } else {
        timestamp.chars().take(12).collect()
    }
}
