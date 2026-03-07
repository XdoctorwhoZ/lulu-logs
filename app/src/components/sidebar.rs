use dioxus::prelude::*;
use dioxus_free_icons::icons::bs_icons::{BsCheck2Square, BsFolder, BsGear, BsHeartPulse, BsTags};
use dioxus_free_icons::Icon;

use crate::app::{export_logs, ActivePanel, AppState};
use crate::components::pulse_panel::PulsePanel;
use crate::components::scenario_panel::ScenarioPanel;

/// Sidebar composed of an ActivityBar (icons) and a collapsible SidePanel.
#[component]
pub fn Sidebar() -> Element {
    let state = use_context::<AppState>();
    let active = *state.active_panel.read();

    rsx! {
        div { class: "sidebar",
            ActivityBar {}
            if active.is_some() {
                SidePanel {}
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Activity Bar (icon strip)
// ---------------------------------------------------------------------------

#[component]
fn ActivityBar() -> Element {
    let state = use_context::<AppState>();
    let active = *state.active_panel.read();

    let sources_count = state.known_sources.read().len();
    let attrs_count = state.known_attributes.read().len();

    let scenarios = state.scenarios.read();
    let pending_count = scenarios
        .iter()
        .filter(|s| {
            matches!(
                s.status,
                crate::models::test_scenario::ScenarioStatus::InProgress
            )
        })
        .count();
    let scenarios_total = scenarios.len();
    drop(scenarios);

    let _tick = *state.pulse_tick.read();
    let pulse = state.pulse_sources.read();
    let online_count = pulse.values().filter(|e| e.is_online()).count();
    drop(pulse);

    rsx! {
        div { class: "activity-bar",
            ActivityIcon {
                panel: ActivePanel::Pulse,
                badge: online_count,
                active: active,
                Icon { icon: BsHeartPulse, width: 20, height: 20 }
            }
            ActivityIcon {
                panel: ActivePanel::Sources,
                badge: sources_count,
                active: active,
                Icon { icon: BsFolder, width: 20, height: 20 }
            }
            ActivityIcon {
                panel: ActivePanel::Attributes,
                badge: attrs_count,
                active: active,
                Icon { icon: BsTags, width: 20, height: 20 }
            }
            ActivityIcon {
                panel: ActivePanel::Scenarios,
                badge: if pending_count > 0 { pending_count } else { scenarios_total },
                active: active,
                Icon { icon: BsCheck2Square, width: 20, height: 20 }
            }
            ActivityIcon {
                panel: ActivePanel::Controls,
                badge: 0,
                active: active,
                Icon { icon: BsGear, width: 20, height: 20 }
            }
        }
    }
}

/// A single icon button in the activity bar.
#[component]
fn ActivityIcon(
    panel: ActivePanel,
    badge: usize,
    active: Option<ActivePanel>,
    children: Element,
) -> Element {
    let mut state = use_context::<AppState>();
    let is_active = active == Some(panel);
    let class = if is_active {
        "activity-icon active"
    } else {
        "activity-icon"
    };

    rsx! {
        div {
            class: "{class}",
            onclick: move |_| {
                let current = *state.active_panel.read();
                if current == Some(panel) {
                    state.active_panel.set(None);
                } else {
                    state.active_panel.set(Some(panel));
                }
            },
            span { class: "activity-icon-symbol", {children} }
            if badge > 0 {
                span { class: "activity-icon-badge", "{badge}" }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Side Panel
// ---------------------------------------------------------------------------

#[component]
fn SidePanel() -> Element {
    let state = use_context::<AppState>();
    let active = *state.active_panel.read();

    let title = match active {
        Some(ActivePanel::Sources) => "Sources",
        Some(ActivePanel::Attributes) => "Attributs",
        Some(ActivePanel::Scenarios) => "Scénarios",
        Some(ActivePanel::Pulse) => "Pulse",
        Some(ActivePanel::Controls) => "Contrôles",
        None => "",
    };

    rsx! {
        div { class: "side-panel",
            div { class: "side-panel-header", "{title}" }
            div { class: "side-panel-content",
                match active {
                    Some(ActivePanel::Sources) => rsx! { SourceFilterPanel {} },
                    Some(ActivePanel::Attributes) => rsx! { AttributeFilterPanel {} },
                    Some(ActivePanel::Scenarios) => rsx! { ScenarioPanel {} },
                    Some(ActivePanel::Pulse) => rsx! { PulsePanel {} },
                    Some(ActivePanel::Controls) => rsx! { ControlsPanel {} },
                    None => rsx! {},
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Filter panels (moved from toolbar.rs)
// ---------------------------------------------------------------------------

/// Source filter panel with text input and checkbox list.
#[component]
fn SourceFilterPanel() -> Element {
    let mut state = use_context::<AppState>();
    let known_sources = state.known_sources.read().clone();
    let hidden_sources = state.hidden_sources.read().clone();
    let filter_text = state.source_filter_text.read().clone();

    let filtered_sources: Vec<String> = known_sources
        .iter()
        .filter(|s| {
            filter_text.is_empty() || s.to_lowercase().contains(&filter_text.to_lowercase())
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "filter-panel",
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
            filter_text.is_empty() || a.to_lowercase().contains(&filter_text.to_lowercase())
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "filter-panel",
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

// ---------------------------------------------------------------------------
// Controls panel (Flow + Actions, moved from toolbar.rs)
// ---------------------------------------------------------------------------

#[component]
fn ControlsPanel() -> Element {
    let mut state = use_context::<AppState>();
    let is_paused = *state.is_paused.read();
    let auto_scroll = *state.auto_scroll.read();

    rsx! {
        div { class: "controls-panel",
            div { class: "controls-section",
                div { class: "controls-section-title", "Flux" }
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
            div { class: "controls-section",
                div { class: "controls-section-title", "Actions" }
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
                        for pin in state.lens_pins.write().iter_mut() {
                            pin.values.clear();
                        }
                    },
                    "🗑 Clear"
                }
            }
        }
    }
}
