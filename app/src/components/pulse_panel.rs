use dioxus::prelude::*;

use crate::app::AppState;

/// Panel showing all sources that have emitted heartbeat pulses,
/// with live online/offline indicators (re-evaluated every second via `pulse_tick`).
#[component]
pub fn PulsePanel() -> Element {
    let state = use_context::<AppState>();

    // Subscribe to pulse_tick so the component re-renders every second.
    // This ensures the online/offline status reflects elapsed time correctly.
    let _tick = state.pulse_tick.read();

    let pulse_sources = state.pulse_sources.read().clone();

    let mut entries: Vec<_> = pulse_sources.values().cloned().collect();
    entries.sort_by(|a, b| a.source.cmp(&b.source));

    let total = entries.len();
    let online_count  = entries.iter().filter(|e| e.is_online()).count();
    let offline_count = total - online_count;

    rsx! {
        div { class: "pulse-panel",
            div { class: "pulse-panel-title",
                "Pulse ({total})"
            }
            if !entries.is_empty() {
                div {
                    style: "font-size: 10px; color: var(--text-muted); padding: 2px 0;",
                    "🟢 {online_count}  🔴 {offline_count}"
                }
            }
            div { class: "pulse-list",
                if entries.is_empty() {
                    div { class: "pulse-empty",
                        "Aucun client détecté"
                    }
                } else {
                    for entry in entries.iter() {
                        {
                            let dot_class = entry.dot_css_class();
                            let label = entry.status_label();
                            let src = entry.source.clone();
                            let ts  = entry.last_seen_ts.clone();
                            let ver = entry.version.clone();
                            rsx! {
                                div { class: "pulse-item",
                                    span { class: "{dot_class}", title: "{label}" }
                                    div { class: "pulse-item-info",
                                        span { class: "pulse-item-name", "{src}" }
                                        if let Some(v) = ver {
                                            span { class: "pulse-item-version", "v{v}" }
                                        }
                                        span { class: "pulse-item-ts",   "{ts}"  }
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
