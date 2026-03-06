use std::collections::{HashMap, HashSet};
use std::time::Instant;

use dioxus::prelude::*;
use flatbuffers;
use rumqttc::{Event, Packet};

use crate::components::{log_list::LogList, lens_view::LensView, sidebar::Sidebar, status_bar::StatusBar};
use crate::generated::lulu_logs_generated::lulu_logs::root_as_log_entry;
use crate::models::{
    decode_data, ActiveView, LensLayout, LensPinData, LuluLevel, LuluLogEntry,
    PulseSourceEntry, ScenarioStatus, TestScenario,
};
use crate::mqtt::PzaMqttClient;

// ---------------------------------------------------------------------------
// Sidebar panel selection
// ---------------------------------------------------------------------------

/// Identifies which panel is displayed in the side panel.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Sources,
    Attributes,
    Scenarios,
    Pulse,
    Controls,
}

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Global reactive state shared across all UI components.
#[derive(Clone, Copy)]
pub struct AppState {
    // --- Data ---
    /// Log entries stored in memory.
    pub logs: Signal<Vec<LuluLogEntry>>,

    // --- Discovered sources and attributes ---
    /// Distinct known sources (sorted alphabetically).
    pub known_sources: Signal<Vec<String>>,
    /// Distinct known attributes (sorted alphabetically).
    pub known_attributes: Signal<Vec<String>>,

    // --- Visibility filters (checkboxes) ---
    /// Set of hidden sources.
    pub hidden_sources: Signal<HashSet<String>>,
    /// Set of hidden attributes.
    pub hidden_attributes: Signal<HashSet<String>>,

    // --- Filters: free text ---
    /// Text filter on the source name (case-insensitive substring match).
    pub source_filter_text: Signal<String>,
    /// Text filter on the attribute name (case-insensitive substring match).
    pub attribute_filter_text: Signal<String>,

    // --- Display controls ---
    /// Flow is paused (new messages are ignored while paused).
    pub is_paused: Signal<bool>,
    /// Auto-scroll to bottom enabled.
    pub auto_scroll: Signal<bool>,

    // --- Connection ---
    /// MQTT client is connected to the broker.
    pub connected: Signal<bool>,

    // --- Counters ---
    /// Total messages received from the broker (across all pauses).
    pub total_received: Signal<usize>,

    // --- Test scenarios (lulu-logs v1.1.0 §3.4) ---
    /// Tracked test scenarios (correlated beg/end pairs).
    pub scenarios: Signal<Vec<TestScenario>>,
    /// When set, only show logs belonging to this scenario (by name+source).
    pub selected_scenario: Signal<Option<(String, String)>>,

    // --- Heartbeats (lulu-logs v1.2.0 §7) ---
    /// Live source entries keyed by source string, updated on each pulse.
    pub pulse_sources: Signal<HashMap<String, PulseSourceEntry>>,
    /// Incremented every second by a background ticker to trigger pulse re-renders.
    pub pulse_tick: Signal<u64>,

    // --- Sidebar ---
    /// Currently active side panel (None = side panel closed).
    pub active_panel: Signal<Option<ActivePanel>>,

    // --- Lens ---
    /// Which view is active in the main area (LogList or Lens).
    pub active_view: Signal<ActiveView>,
    /// Pinned (source, attribute) widgets in the Lens.
    pub lens_pins: Signal<Vec<LensPinData>>,
    /// Current layout preset for Lens widgets.
    pub lens_layout: Signal<LensLayout>,
}

impl AppState {
    /// Creates a new AppState with default values.
    pub fn new() -> Self {
        Self {
            logs: Signal::new(Vec::new()),
            known_sources: Signal::new(Vec::new()),
            known_attributes: Signal::new(Vec::new()),
            hidden_sources: Signal::new(HashSet::new()),
            hidden_attributes: Signal::new(HashSet::new()),
            source_filter_text: Signal::new(String::new()),
            attribute_filter_text: Signal::new(String::new()),
            is_paused: Signal::new(false),
            auto_scroll: Signal::new(true),
            connected: Signal::new(false),
            total_received: Signal::new(0),
            scenarios: Signal::new(Vec::new()),
            selected_scenario: Signal::new(None),
            pulse_sources: Signal::new(HashMap::new()),
            pulse_tick: Signal::new(0u64),
            active_panel: Signal::new(Some(ActivePanel::Sources)),
            active_view: Signal::new(ActiveView::LogList),
            lens_pins: Signal::new(Vec::new()),
            lens_layout: Signal::new(LensLayout::Mosaic),
        }
    }
}

/// Returns `true` if the given log entry passes all 5 visibility filters.
pub fn is_entry_visible(entry: &LuluLogEntry, state: &AppState, log_index: usize) -> bool {
    // 1. Source must not be hidden
    if state.hidden_sources.read().contains(&entry.source) {
        return false;
    }
    // 2. Attribute must not be hidden
    if state.hidden_attributes.read().contains(&entry.attribute) {
        return false;
    }
    // 3. Source text filter
    let src_filter = state.source_filter_text.read();
    if !src_filter.is_empty()
        && !entry
            .source
            .to_lowercase()
            .contains(&src_filter.to_lowercase())
    {
        return false;
    }
    // 4. Attribute text filter
    let attr_filter = state.attribute_filter_text.read();
    if !attr_filter.is_empty()
        && !entry
            .attribute
            .to_lowercase()
            .contains(&attr_filter.to_lowercase())
    {
        return false;
    }
    // 5. Scenario filter — if a scenario is selected, only show logs within its bounds
    if let Some((ref sel_name, ref sel_source)) = *state.selected_scenario.read() {
        let scenarios = state.scenarios.read();
        if let Some(sc) = scenarios
            .iter()
            .find(|s| &s.name == sel_name && &s.source == sel_source)
        {
            if !sc.contains_log_index(log_index) {
                return false;
            }
        } else {
            return false;
        }
    }
    true
}

// ---------------------------------------------------------------------------
// Export .lulu
// ---------------------------------------------------------------------------

/// Exports the currently visible logs to a `.lulu` file.
pub fn export_logs(state: &AppState) {
    use crate::generated::lulu_export_generated::lulu_export::{
        LogRecord, LogRecordArgs, LuluExportFile, LuluExportFileArgs,
    };
    use flatbuffers::FlatBufferBuilder;

    let logs = state.logs.read();
    let visible_logs: Vec<&LuluLogEntry> = logs
        .iter()
        .enumerate()
        .filter(|(idx, e)| is_entry_visible(e, state, *idx))
        .map(|(_, e)| e)
        .collect();

    if visible_logs.is_empty() {
        tracing::warn!("export: no visible logs to export");
        return;
    }

    let mut builder = FlatBufferBuilder::with_capacity(1024 * 1024);

    let records: Vec<_> = visible_logs
        .iter()
        .map(|entry| {
            let topic = builder.create_string(&entry.topic);
            let payload = builder.create_vector(&entry.raw_payload);
            LogRecord::create(
                &mut builder,
                &LogRecordArgs {
                    topic: Some(topic),
                    payload: Some(payload),
                },
            )
        })
        .collect();

    let records_vector = builder.create_vector(&records);
    let export_file = LuluExportFile::create(
        &mut builder,
        &LuluExportFileArgs {
            version: 1,
            records: Some(records_vector),
        },
    );
    builder.finish(export_file, None);
    let bytes = builder.finished_data();

    let filename = chrono::Local::now()
        .format("export_%Y%m%d_%H%M%S.lulu")
        .to_string();
    let path = match std::env::current_dir() {
        Ok(dir) => dir.join(&filename),
        Err(e) => {
            tracing::error!("export: cannot determine current directory: {}", e);
            return;
        }
    };

    match std::fs::write(&path, bytes) {
        Ok(()) => {
            tracing::info!("Exported {} logs to {}", visible_logs.len(), path.display());
        }
        Err(e) => {
            tracing::error!("export: failed to write {}: {}", path.display(), e);
        }
    }
}

// ---------------------------------------------------------------------------
// Root component
// ---------------------------------------------------------------------------

/// Root Dioxus component. Initialises AppState and spawns MQTT listener.
#[component]
pub fn App() -> Element {
    let state = use_hook(AppState::new);

    // Provide AppState to all children via context
    use_context_provider(|| state);

    // Spawn the MQTT listener once
    let _mqtt_resource = use_resource(move || spawn_mqtt_listener(state));

    // Tick every second to drive online/offline transitions in PulsePanel
    let _pulse_ticker = use_resource(move || {
        let mut s = state;
        async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                *s.pulse_tick.write() += 1;
            }
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("assets/style.css") }
        div { class: "app-container",
            div { class: "app-body",
                Sidebar {}
                div { class: "main-area",
                    ViewSwitcher {}
                    match *state.active_view.read() {
                        ActiveView::LogList => rsx! { LogList {} },
                        ActiveView::Lens => rsx! { LensView {} },
                    }
                }
            }
            StatusBar {}
        }
    }
}

/// Tabs to switch between LogList and Lens.
#[component]
fn ViewSwitcher() -> Element {
    let mut state = use_context::<AppState>();
    let active = *state.active_view.read();

    rsx! {
        div { class: "view-switcher",
            button {
                class: if active == ActiveView::LogList { "view-tab active" } else { "view-tab" },
                onclick: move |_| state.active_view.set(ActiveView::LogList),
                "LogList"
            }
            button {
                class: if active == ActiveView::Lens { "view-tab active" } else { "view-tab" },
                onclick: move |_| state.active_view.set(ActiveView::Lens),
                "Lens"
            }
        }
    }
}

/// Background task that connects to the broker and processes incoming messages.
async fn spawn_mqtt_listener(mut state: AppState) {
    let mut mqtt = PzaMqttClient::new("127.0.0.1", 1883);

    // Subscribe to lulu/#
    if let Err(e) = mqtt.subscribe_lulu().await {
        tracing::error!("Failed to subscribe to lulu/#: {}", e);
        return;
    }

    // Subscribe to lulu-pulse/#
    if let Err(e) = mqtt.subscribe_pulse().await {
        tracing::error!("Failed to subscribe to lulu-pulse/#: {}", e);
        return;
    }

    state.connected.set(true);

    loop {
        match mqtt.event_loop.poll().await {
            Ok(Event::Incoming(Packet::Publish(publish))) => {
                let topic = publish.topic.clone();
                let payload_bytes: Vec<u8> = publish.payload.to_vec();

                // Step 1 — Route based on topic prefix
                let segments: Vec<&str> = topic.split('/').collect();

                // --- lulu-pulse/{source…} heartbeat messages ---
                if segments[0] == "lulu-pulse" {
                    if segments.len() < 2 {
                        tracing::warn!("lulu-pulse topic too short: {} — ignoré", topic);
                        continue;
                    }
                    let source = segments[1..].join("/");
                    let json_val = serde_json::from_slice::<serde_json::Value>(&payload_bytes).ok();
                    let ts = json_val
                        .as_ref()
                        .and_then(|v| {
                            v.get("timestamp")
                                .and_then(|t| t.as_str())
                                .map(str::to_string)
                        })
                        .unwrap_or_default();
                    let version = json_val.as_ref().and_then(|v| {
                        v.get("version")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                    });
                    state.pulse_sources.write().insert(
                        source.clone(),
                        PulseSourceEntry {
                            source,
                            last_seen_ts: ts,
                            last_seen_at: Instant::now(),
                            version,
                        },
                    );
                    continue;
                }

                // --- lulu/{source}/{attribute} log messages ---
                // Validate topic (must have at least 3 segments: lulu + source + attr)
                if segments.len() < 3 || segments[0] != "lulu" {
                    tracing::warn!("Topic invalide (< 3 niveaux) : {} — ignoré", topic);
                    continue;
                }

                // Step 2 — Extract source and attribute
                let source = segments[1..segments.len() - 1].join("/");
                let attribute = segments[segments.len() - 1].to_string();

                // Step 3 — Deserialize FlatBuffers
                let log_entry = match root_as_log_entry(&payload_bytes) {
                    Ok(entry) => entry,
                    Err(e) => {
                        tracing::warn!(
                            "FlatBuffers deserialization failed for topic {}: {} — ignoré",
                            topic,
                            e
                        );
                        continue;
                    }
                };

                let timestamp = log_entry.timestamp().to_string();
                let level = LuluLevel::from_fbs(log_entry.level());
                let data_type = log_entry.type_().to_string();
                let data_bytes: Vec<u8> = log_entry.data().iter().collect();
                let decoded_value = decode_data(&data_type, &data_bytes);

                // Step 4 — Update state
                // Always increment total_received (even if paused — actually per spec v2,
                // total_received is NOT incremented during pause)
                if *state.is_paused.read() {
                    continue;
                }

                *state.total_received.write() += 1;

                // Register source
                {
                    let mut sources = state.known_sources.write();
                    if !sources.contains(&source) {
                        sources.push(source.clone());
                        sources.sort();
                    }
                }

                // Register attribute
                {
                    let mut attrs = state.known_attributes.write();
                    if !attrs.contains(&attribute) {
                        attrs.push(attribute.clone());
                        attrs.sort();
                    }
                }

                // Store log entry
                let log_index = {
                    let mut logs = state.logs.write();
                    let idx = logs.len();
                    logs.push(LuluLogEntry {
                        topic,
                        source: source.clone(),
                        attribute: attribute.clone(),
                        timestamp: timestamp.clone(),
                        level,
                        data_type: data_type.clone(),
                        decoded_value: decoded_value.clone(),
                        raw_payload: payload_bytes,
                    });
                    idx
                };

                // Step 5b — Feed Lens pins
                {
                    let mut pins = state.lens_pins.write();
                    for pin in pins.iter_mut() {
                        if pin.matches(&source, &attribute) {
                            pin.push_value(timestamp.clone(), decoded_value.clone());
                        }
                    }
                }

                // Step 5 — Track test scenarios (lulu-logs v1.1.0 §3.4)
                if data_type == "beg_test_scenario" || data_type == "end_test_scenario" {
                    if let Ok(json_val) = serde_json::from_slice::<serde_json::Value>(&data_bytes) {
                        if let Some(scenario_name) = json_val.get("name").and_then(|v| v.as_str()) {
                            let scenario_name = scenario_name.to_string();

                            if data_type == "beg_test_scenario" {
                                state.scenarios.write().push(TestScenario {
                                    name: scenario_name,
                                    source: source.clone(),
                                    attribute: attribute.clone(),
                                    beg_timestamp: timestamp.clone(),
                                    end_timestamp: None,
                                    beg_log_index: log_index,
                                    end_log_index: None,
                                    status: ScenarioStatus::InProgress,
                                });
                            } else {
                                // end_test_scenario — find matching open scenario
                                let success = json_val
                                    .get("success")
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(false);
                                let error_msg = json_val
                                    .get("error")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string();

                                let mut scenarios = state.scenarios.write();
                                if let Some(sc) = scenarios.iter_mut().rev().find(|s| {
                                    s.name == scenario_name
                                        && s.source == source
                                        && s.end_log_index.is_none()
                                }) {
                                    sc.end_timestamp = Some(timestamp.clone());
                                    sc.end_log_index = Some(log_index);
                                    sc.status = if success {
                                        ScenarioStatus::Success
                                    } else {
                                        ScenarioStatus::Failure(error_msg)
                                    };
                                }
                            }
                        }
                    }
                }
            }
            Ok(Event::Incoming(Packet::ConnAck(_))) => {
                state.connected.set(true);
                tracing::info!("MQTT connected to broker");
            }
            Ok(_) => {
                // Other events — ignore
            }
            Err(e) => {
                state.connected.set(false);
                tracing::warn!("MQTT connection error: {} — reconnecting…", e);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                state.connected.set(true);
            }
        }
    }
}
