use dioxus::prelude::*;

use crate::app::{is_entry_visible, AppState};
use crate::components::log_item::LogItem;

/// Scrollable list of visible log entries.
#[component]
pub fn LogList() -> Element {
    let mut state = use_context::<AppState>();
    let logs = state.logs.read();
    let is_paused = *state.is_paused.read();
    let auto_scroll = *state.auto_scroll.read();
    let selected_scenario = state.selected_scenario.read().clone();

    let visible_logs: Vec<_> = logs
        .iter()
        .enumerate()
        .filter(|(idx, entry)| is_entry_visible(entry, &state, *idx))
        .collect();

    let total_count = logs.len();
    let visible_count = visible_logs.len();

    // Auto-scroll effect
    if auto_scroll && !visible_logs.is_empty() {
        document::eval(
            r#"
            const el = document.getElementById('log-list-scroll');
            if (el) { el.scrollTop = el.scrollHeight; }
            "#,
        );
    }

    rsx! {
        if is_paused {
            div { class: "pause-banner",
                "⏸ Flux en pause — les nouveaux messages sont ignorés"
            }
        }
        if let Some((ref sc_name, _)) = selected_scenario {
            div { class: "scenario-filter-banner",
                "🔍 Scénario : {sc_name}"
                span {
                    onclick: move |_| {
                        state.selected_scenario.set(None);
                    },
                    "✕ Effacer le filtre"
                }
            }
        }
        div {
            id: "log-list-scroll",
            class: "log-list-container",
            if total_count == 0 {
                div { class: "log-list-empty",
                    "En attente de messages sur lulu/# …"
                }
            } else if visible_count == 0 {
                div { class: "log-list-empty",
                    "Aucun log ne correspond aux filtres actifs ({visible_count} visible(s) sur {total_count})"
                }
            } else {
                for (idx, entry) in visible_logs {
                    LogItem { key: "{idx}", entry: entry.clone(), log_index: idx }
                }
            }
        }
    }
}
