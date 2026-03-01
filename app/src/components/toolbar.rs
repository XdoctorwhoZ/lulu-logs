use dioxus::prelude::*;

use crate::app::{export_logs, AppState};
use crate::components::scenario_panel::ScenarioPanel;

/// Toolbar component with five sections: Sources, Attributes, Scenarios, Flow Controls, Actions.
#[component]
pub fn Toolbar() -> Element {
    let _state = use_context::<AppState>();

    rsx! {
        div { class: "toolbar",
            // Section: Sources
            SourceFilterPanel {}

            // Section: Attributes
            AttributeFilterPanel {}

            // Section: Test Scenarios
            ScenarioPanel {}

            // Section: Flow controls
            div { class: "toolbar-section",
                div { class: "toolbar-section-title", "Contrôles" }
                FlowControls {}
            }

            // Section: Actions
            div { class: "toolbar-section",
                div { class: "toolbar-section-title", "Actions" }
                ActionButtons {}
            }
        }
    }
}

/// Source filter panel with text input and checkbox list.
#[component]
fn SourceFilterPanel() -> Element {
    let mut state = use_context::<AppState>();
    let known_sources = state.known_sources.read().clone();
    let hidden_sources = state.hidden_sources.read().clone();
    let filter_text = state.source_filter_text.read().clone();

    // Filter the checkbox list by the text input
    let filtered_sources: Vec<String> = known_sources
        .iter()
        .filter(|s| {
            filter_text.is_empty()
                || s.to_lowercase().contains(&filter_text.to_lowercase())
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "filter-panel",
            div { class: "filter-panel-title", "Sources" }
            input {
                class: "filter-text-input",
                r#type: "text",
                placeholder: "Filtrer les sources…",
                value: "{filter_text}",
                oninput: move |evt| {
                    state.source_filter_text.set(evt.value().clone());
                }
            }
            div { class: "filter-bulk-actions",
                span {
                    onclick: move |_| {
                        state.hidden_sources.write().clear();
                    },
                    "Tout afficher"
                }
                span {
                    onclick: move |_| {
                        let all: std::collections::HashSet<String> =
                            state.known_sources.read().iter().cloned().collect();
                        state.hidden_sources.set(all);
                    },
                    "Tout masquer"
                }
            }
            div { class: "filter-checkbox-list",
                for source in filtered_sources {
                    {
                        let source_clone = source.clone();
                        let is_visible = !hidden_sources.contains(&source);
                        rsx! {
                            label { class: "filter-checkbox-item",
                                input {
                                    r#type: "checkbox",
                                    checked: is_visible,
                                    onchange: move |_| {
                                        let mut hidden = state.hidden_sources.write();
                                        if hidden.contains(&source_clone) {
                                            hidden.remove(&source_clone);
                                        } else {
                                            hidden.insert(source_clone.clone());
                                        }
                                    }
                                }
                                "{source}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Attribute filter panel with text input and checkbox list.
#[component]
fn AttributeFilterPanel() -> Element {
    let mut state = use_context::<AppState>();
    let known_attributes = state.known_attributes.read().clone();
    let hidden_attributes = state.hidden_attributes.read().clone();
    let filter_text = state.attribute_filter_text.read().clone();

    let filtered_attributes: Vec<String> = known_attributes
        .iter()
        .filter(|a| {
            filter_text.is_empty()
                || a.to_lowercase().contains(&filter_text.to_lowercase())
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "filter-panel",
            div { class: "filter-panel-title", "Attributs" }
            input {
                class: "filter-text-input",
                r#type: "text",
                placeholder: "Filtrer les attributs…",
                value: "{filter_text}",
                oninput: move |evt| {
                    state.attribute_filter_text.set(evt.value().clone());
                }
            }
            div { class: "filter-bulk-actions",
                span {
                    onclick: move |_| {
                        state.hidden_attributes.write().clear();
                    },
                    "Tout afficher"
                }
                span {
                    onclick: move |_| {
                        let all: std::collections::HashSet<String> =
                            state.known_attributes.read().iter().cloned().collect();
                        state.hidden_attributes.set(all);
                    },
                    "Tout masquer"
                }
            }
            div { class: "filter-checkbox-list",
                for attribute in filtered_attributes {
                    {
                        let attr_clone = attribute.clone();
                        let is_visible = !hidden_attributes.contains(&attribute);
                        rsx! {
                            label { class: "filter-checkbox-item",
                                input {
                                    r#type: "checkbox",
                                    checked: is_visible,
                                    onchange: move |_| {
                                        let mut hidden = state.hidden_attributes.write();
                                        if hidden.contains(&attr_clone) {
                                            hidden.remove(&attr_clone);
                                        } else {
                                            hidden.insert(attr_clone.clone());
                                        }
                                    }
                                }
                                "{attribute}"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Pause/Resume and Auto-scroll toggle buttons.
#[component]
fn FlowControls() -> Element {
    let mut state = use_context::<AppState>();
    let is_paused = *state.is_paused.read();
    let auto_scroll = *state.auto_scroll.read();

    rsx! {
        button {
            class: if is_paused { "toolbar-btn btn-paused" } else { "toolbar-btn btn-playing" },
            onclick: move |_| {
                let current = *state.is_paused.read();
                state.is_paused.set(!current);
            },
            if is_paused { "▶ Resume" } else { "⏸ Pause" }
        }
        button {
            class: "toolbar-btn",
            onclick: move |_| {
                let current = *state.auto_scroll.read();
                state.auto_scroll.set(!current);
            },
            if auto_scroll { "⬇ Auto-scroll ON" } else { "⬇ Auto-scroll OFF" }
        }
    }
}

/// Export and Clear buttons.
#[component]
fn ActionButtons() -> Element {
    let mut state = use_context::<AppState>();

    rsx! {
        button {
            class: "toolbar-btn btn-export",
            onclick: move |_| {
                export_logs(&state);
            },
            "📥 Export .lulu"
        }
        button {
            class: "toolbar-btn btn-clear",
            onclick: move |_| {
                state.logs.write().clear();
                state.total_received.set(0);
            },
            "🗑 Clear"
        }
    }
}
