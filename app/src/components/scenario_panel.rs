use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::test_scenario::ScenarioStatus;

/// Panel listing all tracked test scenarios with status badges.
/// Clicking a scenario filters the log list to show only its logs.
#[component]
pub fn ScenarioPanel() -> Element {
    let mut state = use_context::<AppState>();
    let scenarios = state.scenarios.read().clone();
    let selected = state.selected_scenario.read().clone();

    let total = scenarios.len();
    let passed = scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Success)).count();
    let failed = scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::Failure(_))).count();
    let pending = scenarios.iter().filter(|s| matches!(s.status, ScenarioStatus::InProgress)).count();

    rsx! {
        div { class: "scenario-panel",
            div { class: "scenario-panel-title",
                "Scénarios ({total})"
            }
            if !scenarios.is_empty() {
                div {
                    style: "font-size: 10px; color: var(--text-muted); padding: 2px 0;",
                    "✅ {passed}  ❌ {failed}  ⏳ {pending}"
                }
            }
            div { class: "scenario-list",
                if scenarios.is_empty() {
                    div { class: "scenario-empty",
                        "Aucun scénario détecté"
                    }
                } else {
                    for sc in scenarios.iter().rev() {
                        {
                            let sc_name = sc.name.clone();
                            let sc_source = sc.source.clone();
                            let is_selected = selected.as_ref()
                                .is_some_and(|(n, s)| n == &sc_name && s == &sc_source);
                            let item_class = if is_selected {
                                "scenario-item selected"
                            } else {
                                "scenario-item"
                            };
                            let badge_class = sc.status_css_class();
                            let label = sc.status_label();
                            let error_msg = sc.error_message().map(|s| s.to_string());

                            rsx! {
                                div {
                                    class: "{item_class}",
                                    onclick: {
                                        let name = sc_name.clone();
                                        let source = sc_source.clone();
                                        move |_| {
                                            let current = state.selected_scenario.read().clone();
                                            if current.as_ref().is_some_and(|(n, s)| n == &name && s == &source) {
                                                state.selected_scenario.set(None);
                                            } else {
                                                state.selected_scenario.set(Some((name.clone(), source.clone())));
                                            }
                                        }
                                    },
                                    div { class: "scenario-item-header",
                                        span { class: "scenario-item-name", "{sc_name}" }
                                        span { class: "{badge_class}", "{label}" }
                                    }
                                    if let Some(err) = error_msg {
                                        div { class: "scenario-error-text",
                                            "{err}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
