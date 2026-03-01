use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::test_scenario::ScenarioStatus;

/// Bottom status bar showing connection state and counters.
#[component]
pub fn StatusBar() -> Element {
    let state = use_context::<AppState>();
    let connected = *state.connected.read();
    let logs_count = state.logs.read().len();
    let total_received = *state.total_received.read();
    let sources_count = state.known_sources.read().len();
    let attributes_count = state.known_attributes.read().len();

    let scenarios = state.scenarios.read();
    let sc_total = scenarios.len();
    let sc_pass = scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Success)).count();
    let sc_fail = scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Failure(_))).count();
    let sc_run = scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::InProgress)).count();
    drop(scenarios);

    let (dot_class, label) = if connected {
        ("status-dot connected", "Connected")
    } else {
        ("status-dot disconnected", "Disconnected")
    };

    rsx! {
        div { class: "status-bar",
            div { class: "status-indicator",
                span { class: "{dot_class}" }
                span { class: "status-label", "Broker: {label}" }
            }
            span { class: "status-label",
                "Logs: "
                span { class: "status-count", "{logs_count}" }
            }
            span { class: "status-label",
                "Total reçus: "
                span { class: "status-count", "{total_received}" }
            }
            span { class: "status-label",
                "Sources: "
                span { class: "status-count", "{sources_count}" }
            }
            span { class: "status-label",
                "Attributs: "
                span { class: "status-attr-badge", "{attributes_count}" }
            }
            if sc_total > 0 {
                span { class: "status-label",
                    "Scénarios: "
                    span { class: "scenario-badge-success", "{sc_pass}" }
                    " / "
                    span { class: "scenario-badge-fail", "{sc_fail}" }
                    " / "
                    span { class: "scenario-badge-pending", "{sc_run}" }
                }
            }
        }
    }
}
