use dioxus::prelude::*;

use crate::app::AppState;
use crate::models::lens_pin::{LensLayout, LensPinData};

/// Root Lens view — displays pinned attribute widgets in a configurable grid.
#[component]
pub fn LensView() -> Element {
    let state = use_context::<AppState>();
    let pins = state.lens_pins.read();
    let layout = *state.lens_layout.read();

    rsx! {
        div { class: "lens-container",
            LensHeader {}
            if pins.is_empty() {
                div { class: "lens-empty",
                    "Aucun attribut épinglé. Faites un clic droit sur une entrée de la LogList pour épingler un attribut."
                }
            } else {
                div { class: "{layout.css_class()}",
                    for (idx , pin) in pins.iter().enumerate() {
                        LensWidget { key: "{idx}", pin: pin.clone(), index: idx }
                    }
                }
            }
        }
    }
}

/// Header bar with layout selector.
#[component]
fn LensHeader() -> Element {
    let mut state = use_context::<AppState>();
    let current = *state.lens_layout.read();

    let layouts = [
        LensLayout::Column,
        LensLayout::Grid2,
        LensLayout::Grid3,
        LensLayout::Mosaic,
    ];

    rsx! {
        div { class: "lens-header",
            span { class: "lens-header-title", "LENS" }
            div { class: "lens-layout-selector",
                for layout in layouts {
                    button {
                        class: if current == layout { "lens-layout-btn active" } else { "lens-layout-btn" },
                        onclick: move |_| state.lens_layout.set(layout),
                        "{layout.label()}"
                    }
                }
            }
        }
    }
}

/// A single pinned attribute widget.
#[component]
fn LensWidget(pin: LensPinData, index: usize) -> Element {
    let mut state = use_context::<AppState>();

    let last_value = pin.values.back();
    let relative_time = last_value.map(|v| format_relative_time(&v.timestamp)).unwrap_or_default();
    let last_raw = last_value.map(|v| v.raw.as_str()).unwrap_or("—");

    rsx! {
        div { class: "lens-widget",
            // Header
            div { class: "lens-widget-header",
                span { class: "lens-widget-source", "{pin.source}" }
                span { class: "lens-widget-sep", " / " }
                span { class: "lens-widget-attr", "{pin.attribute}" }
                button {
                    class: "lens-widget-close",
                    onclick: move |_| {
                        state.lens_pins.write().remove(index);
                    },
                    "✕"
                }
            }
            // Body — dispatch by data_type
            div { class: "lens-widget-body",
                {match pin.data_type.as_str() {
                    "float32" | "float64" | "int32" | "int64" => rsx! { SparklineWidget { pin: pin.clone() } },
                    "bool" => rsx! { BoolTimelineWidget { pin: pin.clone() } },
                    "string" | "json" => rsx! { TextHistoryWidget { pin: pin.clone() } },
                    _ => rsx! { PlaceholderWidget { pin: pin.clone() } },
                }}
            }
            // Footer
            div { class: "lens-widget-footer",
                span { class: "lens-widget-last-val", "{last_raw}" }
                if !relative_time.is_empty() {
                    span { class: "lens-widget-time", "{relative_time}" }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Sub-widgets
// ---------------------------------------------------------------------------

/// Sparkline — inline SVG polyline from numeric values.
#[component]
fn SparklineWidget(pin: LensPinData) -> Element {
    let values: Vec<f64> = pin
        .values
        .iter()
        .filter_map(|v| v.raw.trim().parse::<f64>().ok())
        .collect();

    if values.is_empty() {
        return rsx! { div { class: "lens-widget-placeholder", "En attente de données…" } };
    }

    let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = if (max - min).abs() < 1e-9 { 1.0 } else { max - min };

    let width = 300.0_f64;
    let height = 80.0_f64;
    let padding = 2.0_f64;
    let usable_h = height - 2.0 * padding;
    let step = if values.len() > 1 {
        width / (values.len() - 1) as f64
    } else {
        width
    };

    let points: String = values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let x = i as f64 * step;
            let y = padding + usable_h - ((v - min) / range) * usable_h;
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");

    rsx! {
        svg {
            class: "sparkline-svg",
            view_box: "0 0 {width} {height}",
            polyline {
                points: "{points}",
                fill: "none",
                stroke: "var(--accent)",
                stroke_width: "1.5",
            }
        }
    }
}

/// Boolean timeline — horizontal colored bar of true/false segments.
#[component]
fn BoolTimelineWidget(pin: LensPinData) -> Element {
    if pin.values.is_empty() {
        return rsx! { div { class: "lens-widget-placeholder", "En attente de données…" } };
    }

    rsx! {
        div { class: "bool-timeline",
            for val in pin.values.iter() {
                {
                    let is_true = matches!(val.raw.trim(), "true" | "1");
                    let cls = if is_true { "bool-seg bool-true" } else { "bool-seg bool-false" };
                    rsx! { div { class: "{cls}" } }
                }
            }
        }
    }
}

/// Text history — scrollable list of recent values.
#[component]
fn TextHistoryWidget(pin: LensPinData) -> Element {
    if pin.values.is_empty() {
        return rsx! { div { class: "lens-widget-placeholder", "En attente de données…" } };
    }

    rsx! {
        div { class: "text-history",
            for val in pin.values.iter().rev() {
                div { class: "text-history-item",
                    span { class: "text-history-ts", "{extract_time(&val.timestamp)}" }
                    span { class: "text-history-val", "{val.raw}" }
                }
            }
        }
    }
}

/// Placeholder for unsupported data types.
#[component]
fn PlaceholderWidget(pin: LensPinData) -> Element {
    rsx! {
        div { class: "lens-widget-placeholder",
            "{pin.source} / {pin.attribute}"
            br {}
            "Type non pris en charge ({pin.data_type})"
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extracts `HH:MM:SS` from an ISO 8601 timestamp.
fn extract_time(ts: &str) -> String {
    if let Some(t_pos) = ts.find('T') {
        let time_part = &ts[t_pos + 1..];
        let end = time_part
            .find('Z')
            .or_else(|| time_part.find('+'))
            .unwrap_or(time_part.len())
            .min(8);
        time_part[..end].to_string()
    } else {
        ts.chars().take(8).collect()
    }
}

/// Formats a very rough relative timestamp (e.g. "il y a 3 s").
fn format_relative_time(ts: &str) -> String {
    let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) else {
        return String::new();
    };
    let now = chrono::Utc::now();
    let diff = now.signed_duration_since(parsed);
    let secs = diff.num_seconds();
    if secs < 2 {
        "à l'instant".to_string()
    } else if secs < 60 {
        format!("il y a {} s", secs)
    } else if secs < 3600 {
        format!("il y a {} min", secs / 60)
    } else {
        format!("il y a {} h", secs / 3600)
    }
}
