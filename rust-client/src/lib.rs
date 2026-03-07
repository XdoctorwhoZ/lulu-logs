//! # lulu-logs-client
//!
//! MQTT client library for the **lulu-logs** protocol.
//!
//! This crate provides a singleton API (`lulu_init`, `lulu_publish`, `lulu_shutdown`)
//! that serialises log entries as FlatBuffers payloads and publishes them over MQTT.

use std::future::Future;
use std::sync::OnceLock;
use std::time::Duration;

mod client;
mod error;
mod models;
mod rand_util;
mod serializer;
mod topic;

#[allow(dead_code, unused_imports, clippy::all)]
mod lulu_logs_generated;

// --- Public re-exports ---
pub use client::{LuluClientConfig, LuluStats};
pub use error::LuluError;
pub use models::{Data, DataType, LogLevel};

use client::LuluClient;
use serializer::PendingMessage;

// ---------------------------------------------------------------------------
// Singletons
// ---------------------------------------------------------------------------

static GLOBAL_CLIENT: OnceLock<LuluClient> = OnceLock::new();
static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Returns (or lazily creates) the dedicated single-threaded tokio runtime.
pub(crate) fn get_or_init_runtime() -> &'static tokio::runtime::Runtime {
    TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .thread_name("lulu-logs-client-rt")
            .enable_all()
            .build()
            .expect("failed to create lulu-logs-client tokio runtime")
    })
}

/// Drives `fut` to completion, working whether or not the calling thread is
/// already inside a Tokio runtime.
///
/// * **Inside a multi-thread runtime** (`#[tokio::main]`, test harness, …):
///   uses [`tokio::task::block_in_place`] so we can block without holding the
///   runtime scheduler.
/// * **No runtime on this thread**: falls back to the crate-local dedicated
///   runtime (`TOKIO_RUNTIME`).
///
/// # Panics
/// Panics if called from inside a *current-thread* (`basic_scheduler`) runtime,
/// because `block_in_place` requires a multi-thread runtime.
fn block_on_smart<F>(fut: F) -> F::Output
where
    F: Future,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(fut)),
        Err(_) => get_or_init_runtime().block_on(fut),
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Initialises the global lulu-logs-client singleton.
///
/// Must be called exactly once before any `lulu_publish()`. Calling it a second
/// time returns `Err(LuluError::AlreadyInitialized)`.
pub fn lulu_init(config: LuluClientConfig) -> Result<(), LuluError> {
    if GLOBAL_CLIENT.get().is_some() {
        return Err(LuluError::AlreadyInitialized);
    }

    let client =
        block_on_smart(LuluClient::start(config)).map_err(|_| LuluError::AlreadyInitialized)?;

    GLOBAL_CLIENT
        .set(client)
        .map_err(|_| LuluError::AlreadyInitialized)
}

/// Publishes a log entry onto the MQTT bus.
///
/// The message is validated, enqueued, then serialised and published
/// asynchronously by the background send-loop.
pub fn lulu_publish(
    source: &str,
    attribute: &str,
    level: LogLevel,
    data: Data,
) -> Result<(), LuluError> {
    let client = GLOBAL_CLIENT.get().ok_or(LuluError::NotInitialized)?;

    let source_segments = topic::parse_source(source)?;
    topic::validate_attribute(attribute)?;

    client.publish(PendingMessage {
        source_segments,
        attribute: attribute.to_string(),
        level,
        data,
    })
}

/// Gracefully shuts down the lulu-logs-client.
///
/// Waits up to 5 seconds for the internal queue to drain before returning.
/// If the client was never initialised, this function returns immediately.
pub fn lulu_shutdown() {
    let client = match GLOBAL_CLIENT.get() {
        Some(c) => c,
        None => return,
    };

    // Stop all heartbeat tasks before draining the log queue.
    client.stop_all_pulses();

    let _ = block_on_smart(async {
        tokio::time::timeout(Duration::from_secs(5), async {
            loop {
                if client.stats().queue_current_size == 0 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
    });

    // OnceLock cannot be reset — the connection will close when the process exits.
    // If init → shutdown → init cycles are required, replace OnceLock<LuluClient>
    // with Mutex<Option<LuluClient>>.
}

/// Returns `true` if `lulu_init()` has been called successfully.
pub fn lulu_is_initialized() -> bool {
    GLOBAL_CLIENT.get().is_some()
}

/// Returns `true` if the MQTT connection is currently alive.
pub fn lulu_is_connected() -> bool {
    GLOBAL_CLIENT
        .get()
        .map(|c| c.is_connected())
        .unwrap_or(false)
}

/// Returns runtime statistics, or `None` if the client is not initialised.
pub fn lulu_stats() -> Option<LuluStats> {
    GLOBAL_CLIENT.get().map(|c| c.stats())
}

// ---------------------------------------------------------------------------
// Heartbeat helpers (lulu-logs v1.2.0 §7)
// ---------------------------------------------------------------------------

/// Starts a background heartbeat on `lulu-pulse/{source}` every 2 seconds.
///
/// The first pulse is emitted immediately upon registration. Calling again
/// with the same source replaces the existing task (idempotent).
///
/// # Errors
/// Returns `LuluError::NotInitialized` if `lulu_init()` has not been called,
/// or `LuluError::InvalidSource` if `source` contains invalid segments.
pub fn lulu_start_pulse(source: &str, version: Option<&str>) -> Result<(), LuluError> {
    let client = GLOBAL_CLIENT.get().ok_or(LuluError::NotInitialized)?;
    let source_segments = topic::parse_source(source)?;
    let pulse_topic = topic::build_pulse_topic(&source_segments);
    let rt = get_or_init_runtime();
    client.start_pulse(source.to_string(), pulse_topic, version.map(str::to_string), rt);
    Ok(())
}

/// Stops the heartbeat for the given source. No-op if no pulse is running
/// for that source, or if the client is not initialised.
pub fn lulu_stop_pulse(source: &str) {
    if let Some(client) = GLOBAL_CLIENT.get() {
        client.stop_pulse(source);
    }
}

// ---------------------------------------------------------------------------
// Test scenario convenience helpers (lulu-logs v1.1.0 §3.4)
// ---------------------------------------------------------------------------

/// Publishes a `beg_test_scenario` log entry marking the start of a named test scenario.
///
/// The `data` payload is a JSON object `{"name":"<scenario_name>"}` encoded as UTF-8 bytes.
/// The log level is always `Info`.
pub fn lulu_beg_test_scenario(
    source: &str,
    attribute: &str,
    scenario_name: &str,
) -> Result<(), LuluError> {
    let json = format!(r#"{{"name":"{}"}}"#, scenario_name);
    lulu_publish(
        source,
        attribute,
        LogLevel::Info,
        Data::BegTestScenario(json),
    )
}

/// Publishes an `end_test_scenario` log entry marking the end of a named test scenario.
///
/// The `data` payload is a JSON object containing `name`, `success`, and (on failure) `error`.
/// The log level is `Info` when `success` is `true`, `Error` when `false`.
///
/// # Arguments
/// * `source` — MQTT source segments (e.g. `"mcp/filesystem"`)
/// * `attribute` — MQTT attribute (last topic segment)
/// * `scenario_name` — must match the name used in the corresponding `lulu_beg_test_scenario`
/// * `success` — `true` = scenario passed, `false` = scenario failed
/// * `error` — required when `success` is `false`; human-readable failure description
pub fn lulu_end_test_scenario(
    source: &str,
    attribute: &str,
    scenario_name: &str,
    success: bool,
    error: Option<&str>,
) -> Result<(), LuluError> {
    let json = if success {
        format!(r#"{{"name":"{}","success":true}}"#, scenario_name)
    } else {
        let err_msg = error.unwrap_or("unknown error");
        format!(
            r#"{{"name":"{}","success":false,"error":"{}"}}"#,
            scenario_name,
            err_msg.replace('\\', "\\\\").replace('"', "\\\"")
        )
    };

    let level = if success {
        LogLevel::Info
    } else {
        LogLevel::Error
    };
    lulu_publish(source, attribute, level, Data::EndTestScenario(json))
}
