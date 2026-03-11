# lulu-logs-client — User Manual

> **Protocol version**: lulu-logs v1.2.0  
> **Crate**: `lulu-logs-client` v0.1.0  
> **Target audience**: AI agents and developers integrating lulu-logs via Rust.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Adding the dependency](#2-adding-the-dependency)
3. [Concepts](#3-concepts)
4. [Lifecycle](#4-lifecycle)
5. [Configuration reference](#5-configuration-reference)
6. [Publishing log entries](#6-publishing-log-entries)
7. [Data types reference](#7-data-types-reference)
8. [Topic naming rules](#8-topic-naming-rules)
9. [Log levels](#9-log-levels)
10. [Heartbeat (pulse)](#10-heartbeat-pulse)
11. [Test scenario helpers](#11-test-scenario-helpers)
12. [Error handling](#12-error-handling)
13. [Runtime statistics](#13-runtime-statistics)
14. [Complete worked example](#14-complete-worked-example)
15. [lulu-inject binary](#15-lulu-inject-binary)
16. [Constraints and limits](#16-constraints-and-limits)
17. [Quick-reference cheat-sheet](#17-quick-reference-cheat-sheet)

---

## 1. Overview

`lulu-logs-client` is a singleton MQTT client library that serialises structured log entries as [FlatBuffers](https://flatbuffers.dev/) payloads and publishes them over MQTT (QoS 0).

Key design choices relevant to any consumer:

| Property | Value |
|---|---|
| MQTT broker | configurable, default `127.0.0.1:1883` |
| QoS | 0 (at-most-once) |
| Retain | false |
| Payload format | FlatBuffers `LogEntry` (≤ 20 480 bytes) |
| Timestamps | ISO 8601 UTC with millisecond precision |
| Threading | single background Tokio runtime thread; all public functions are sync |
| Singleton | one global client per process (`OnceLock`) |

---

## 2. Adding the dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
lulu-logs-client = { path = "../rust-client" }
```

No feature flags are required. The crate brings its own Tokio runtime internally, so it works whether or not the calling crate already uses Tokio.

---

## 3. Concepts

### Source

A **source** identifies the hardware or software component emitting logs. It is a `/`-separated path of one or more segments:

```
psu/channel-1
oscilloscope/probe-a
my-service
```

Each segment must contain only ASCII alphanumerics, hyphens (`-`), or underscores (`_`). No empty segments, no trailing slash.

### Attribute

An **attribute** names the specific measurement or event within the source — e.g. `voltage`, `status`, `report`. It must be a single non-empty string with no `/`.

### Topic

The MQTT topic is assembled automatically:

```
lulu/{source}/{attribute}
```

Example: source `psu/channel-1`, attribute `voltage` → topic `lulu/psu/channel-1/voltage`.

The source and attribute are **not** stored inside the FlatBuffers payload; they live exclusively in the topic.

---

## 4. Lifecycle

### Initialise

Must be called exactly once before any publish:

```rust
use lulu_logs_client::{lulu_init, LuluClientConfig};

lulu_init(LuluClientConfig::default())?;
```

Returns `Err(LuluError::AlreadyInitialized)` if called a second time.

### Check status

```rust
use lulu_logs_client::{lulu_is_initialized, lulu_is_connected};

if lulu_is_initialized() { /* safe to publish */ }
if lulu_is_connected()   { /* broker reachable */ }
```

### Shutdown

Call at the end of the process to drain the queue gracefully (waits up to 5 s):

```rust
use lulu_logs_client::lulu_shutdown;

lulu_shutdown();
```

`lulu_shutdown` is idempotent — calling it before `lulu_init` is safe.

> **Note**: the current implementation uses `OnceLock`, so init → shutdown → reinit cycles in the same process are not supported. If that pattern is needed, a `Mutex<Option<LuluClient>>` upgrade is required.

---

## 5. Configuration reference

```rust
use lulu_logs_client::LuluClientConfig;

let config = LuluClientConfig {
    broker_host:        "192.168.1.10".to_string(), // MQTT broker IP or hostname
    broker_port:        1883,                        // MQTT broker port
    client_id_prefix:   "my-app".to_string(),        // prefix; a 6-char random suffix is appended
    queue_capacity:     256,                         // internal channel capacity (messages)
    keep_alive_secs:    5,                           // MQTT keep-alive interval
};
```

| Field | Default | Notes |
|---|---|---|
| `broker_host` | `"127.0.0.1"` | Any reachable hostname or IP |
| `broker_port` | `1883` | Standard MQTT plaintext port |
| `client_id_prefix` | `"lulu-logs-client"` | Final client ID = `{prefix}-{6 random chars}` |
| `queue_capacity` | `256` | Extra publishes while full → `LuluError::QueueFull` |
| `keep_alive_secs` | `5` | Passed directly to `rumqttc` |

`LuluClientConfig::default()` uses all the values in the default column above.

---

## 6. Publishing log entries

```rust
use lulu_logs_client::{lulu_publish, LogLevel, Data};

lulu_publish(source, attribute, level, data)?;
```

| Parameter | Type | Description |
|---|---|---|
| `source` | `&str` | Source path (`"psu/channel-1"`) |
| `attribute` | `&str` | Attribute name (`"voltage"`) |
| `level` | `LogLevel` | Severity (see §9) |
| `data` | `Data` | Typed value (see §7) |

The function validates source and attribute, enqueues the message, and returns immediately. Serialisation and MQTT publish happen asynchronously on the background thread.

### Minimal example

```rust
lulu_publish("my-service", "status", LogLevel::Info, Data::String("ready".into()))?;
```

---

## 7. Data types reference

All variants of the `Data` enum:

| Variant | `type` string on wire | Encoding | Rust example |
|---|---|---|---|
| `Data::String(s)` | `"string"` | UTF-8 bytes | `Data::String("hello".into())` |
| `Data::Int32(v)` | `"int32"` | 4 bytes little-endian | `Data::Int32(-42)` |
| `Data::Int64(v)` | `"int64"` | 8 bytes little-endian | `Data::Int64(1_000_000)` |
| `Data::Float32(v)` | `"float32"` | 4 bytes IEEE 754 LE | `Data::Float32(3.3_f32)` |
| `Data::Float64(v)` | `"float64"` | 8 bytes IEEE 754 LE | `Data::Float64(50.0_f64)` |
| `Data::Bool(v)` | `"bool"` | 1 byte (`0x00` / `0x01`) | `Data::Bool(true)` |
| `Data::Json(s)` | `"json"` | UTF-8 JSON string | `Data::Json(r#"{"k":1}"#.into())` |
| `Data::Bytes(v)` | `"bytes"` | Raw binary | `Data::Bytes(vec![0xDE, 0xAD])` |
| `Data::BegTestScenario(json)` | `"beg_test_scenario"` | UTF-8 JSON `{"name":"…"}` | see §11 |
| `Data::EndTestScenario(json)` | `"end_test_scenario"` | UTF-8 JSON `{"name","success"[,"error"]}` | see §11 |

### Type selection guide

- Numeric measurements → `Float32` / `Float64` (physical quantities), `Int32` / `Int64` (counters, addresses).
- On/off, pass/fail signals → `Bool`.
- Human-readable messages, identifiers → `String`.
- Structured objects or arrays → `Json` (valid JSON required by convention).
- Raw serial/binary frames → `Bytes`.
- Test lifecycle events → `BegTestScenario` / `EndTestScenario` (use helpers in §11).

---

## 8. Topic naming rules

### Source segments

- One or more segments separated by `/`.
- Each segment: non-empty, only `[A-Za-z0-9\-_]`.
- Examples: `psu`, `psu/channel-1`, `lab/bench-3/probe-a`.

### Attribute

- Single segment (no `/`), non-empty.
- Any printable characters except `/`.
- Examples: `voltage`, `read-file`, `status_raw`.

### Pulse topic (auto-built)

```
lulu-pulse/{source}
```

Example: source `psu/channel-1` → pulse topic `lulu-pulse/psu/channel-1`.

### Validation errors

| Error | Cause |
|---|---|
| `LuluError::InvalidSource` | Empty segment, trailing `/`, or forbidden character |
| `LuluError::InvalidAttribute` | Empty string or contains `/` |

---

## 9. Log levels

```rust
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info  = 2,   // default in FlatBuffers schema
    Warn  = 3,
    Error = 4,
    Fatal = 5,
}
```

Use the level that matches the semantic severity of the event:

| Level | When to use |
|---|---|
| `Trace` | Fine-grained diagnostic data, high-frequency sampling |
| `Debug` | Internal state dumps, intermediate computations |
| `Info` | Normal operational events (startup, configuration) |
| `Warn` | Recoverable issues, out-of-range values |
| `Error` | Failures that affect a single operation |
| `Fatal` | Unrecoverable failures, hardware faults |

---

## 10. Heartbeat (pulse)

A **pulse** is a periodic 2-second heartbeat published on `lulu-pulse/{source}`. The UI uses it to display online/offline status (offline threshold: 6 s = 3 × interval).

### Start a pulse

```rust
use lulu_logs_client::lulu_start_pulse;

// With version string:
lulu_start_pulse("psu/channel-1", Some("1.0.0"))?;

// Without version:
lulu_start_pulse("my-service", None)?;
```

The first pulse is emitted immediately upon registration. Calling `lulu_start_pulse` again for the same source replaces the existing task (idempotent).

Pulse payload (raw JSON, not FlatBuffers):

```json
{"timestamp": "2026-03-11T10:00:00.000Z", "version": "1.0.0"}
// or, without version:
{"timestamp": "2026-03-11T10:00:00.000Z"}
```

### Stop a pulse

```rust
use lulu_logs_client::lulu_stop_pulse;

lulu_stop_pulse("psu/channel-1");
```

All active pulses are stopped automatically by `lulu_shutdown()`.

---

## 11. Test scenario helpers

Test scenarios provide structured begin/end markers that the UI groups into collapsible test run summaries.

### Begin a scenario

```rust
use lulu_logs_client::lulu_beg_test_scenario;

lulu_beg_test_scenario("session/run-001", "control", "voltage-regulation-3v3")?;
```

This publishes on topic `lulu/session/run-001/control` with:

```json
{"name": "voltage-regulation-3v3"}
```

level: `Info`, type: `beg_test_scenario`.

### End a scenario – success

```rust
use lulu_logs_client::lulu_end_test_scenario;

lulu_end_test_scenario("session/run-001", "control", "voltage-regulation-3v3", true, None)?;
```

Payload:
```json
{"name": "voltage-regulation-3v3", "success": true}
```

level: `Info`.

### End a scenario – failure

```rust
lulu_end_test_scenario(
    "session/run-001",
    "control",
    "overcurrent-protection",
    false,
    Some("current limit not triggered at 2.1 A"),
)?;
```

Payload:
```json
{"name": "overcurrent-protection", "success": false, "error": "current limit not triggered at 2.1 A"}
```

level: `Error`.

### Using `lulu_publish` directly

The helpers above are thin wrappers. Equivalent manual call for begin:

```rust
lulu_publish(
    "session/run-001",
    "control",
    LogLevel::Info,
    Data::BegTestScenario(r#"{"name":"voltage-regulation-3v3"}"#.into()),
)?;
```

---

## 12. Error handling

All public functions that can fail return `Result<_, LuluError>`.

```rust
pub enum LuluError {
    AlreadyInitialized,   // lulu_init() called more than once
    NotInitialized,       // lulu_publish/pulse called before lulu_init()
    QueueFull,            // internal channel at capacity, message dropped
    InvalidSource(String),    // source string failed validation
    InvalidAttribute(String), // attribute string failed validation
    InvalidType,          // (reserved, not currently returned)
}
```

### Recommended error handling strategy

```rust
use lulu_logs_client::LuluError;

match lulu_publish("src", "attr", LogLevel::Info, Data::Bool(true)) {
    Ok(()) => {}
    Err(LuluError::QueueFull) => {
        // Back-pressure: slow down or drop non-critical messages.
    }
    Err(LuluError::NotInitialized) => {
        // Bug: publish called before init.
        panic!("lulu not initialised");
    }
    Err(e) => {
        eprintln!("lulu_publish error: {e}");
    }
}
```

`QueueFull` is the only error that can appear at steady-state runtime (all others are programming errors). A queue capacity of 256 is generous for low-frequency telemetry; increase `LuluClientConfig::queue_capacity` if you publish in bursts.

---

## 13. Runtime statistics

```rust
use lulu_logs_client::lulu_stats;

if let Some(stats) = lulu_stats() {
    println!("published:    {}", stats.messages_published);
    println!("dropped:      {}", stats.messages_dropped);
    println!("queue depth:  {}", stats.queue_current_size);
    println!("reconnects:   {}", stats.reconnections);
}
```

| Field | Description |
|---|---|
| `messages_published` | Total successfully sent to broker |
| `messages_dropped` | Total dropped (queue full or serialisation error) |
| `queue_current_size` | Number of messages currently waiting in queue |
| `reconnections` | Number of broker reconnection cycles |

Returns `None` if `lulu_init()` has not been called.

---

## 14. Complete worked example

```rust
use lulu_logs_client::{
    lulu_init, lulu_publish, lulu_start_pulse, lulu_stop_pulse,
    lulu_beg_test_scenario, lulu_end_test_scenario,
    lulu_stats, lulu_shutdown,
    Data, LogLevel, LuluClientConfig,
};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialise (connect to local broker)
    lulu_init(LuluClientConfig::default())?;

    // 2. Start heartbeat for the source
    lulu_start_pulse("bench/psu", Some("2.0.0"))?;

    // 3. Publish various data types
    lulu_publish("bench/psu", "voltage",  LogLevel::Info,  Data::Float32(3.295))?;
    lulu_publish("bench/psu", "current",  LogLevel::Info,  Data::Float32(0.412))?;
    lulu_publish("bench/psu", "enabled",  LogLevel::Info,  Data::Bool(true))?;
    lulu_publish("bench/psu", "status",   LogLevel::Debug, Data::String("CV".into()))?;
    lulu_publish("bench/psu", "report",   LogLevel::Info,
        Data::Json(r#"{"mode":"CV","setpoint":3.3,"actual":3.295}"#.into()))?;
    lulu_publish("bench/psu", "raw-frame", LogLevel::Trace, Data::Bytes(vec![0x02, 0x41, 0x03]))?;

    // 4. Simulate a test scenario
    lulu_beg_test_scenario("bench/psu", "ctrl", "output-accuracy")?;

    let measured = 3.295_f32;
    let ok = (measured - 3.3).abs() < 0.05;

    lulu_end_test_scenario(
        "bench/psu", "ctrl", "output-accuracy",
        ok,
        if ok { None } else { Some("Voltage out of ±50 mV tolerance") },
    )?;

    // 5. Brief pause so the pulse fires at least once
    thread::sleep(Duration::from_secs(3));

    // 6. Stop the pulse
    lulu_stop_pulse("bench/psu");

    // 7. Print stats
    if let Some(s) = lulu_stats() {
        println!("published={} dropped={}", s.messages_published, s.messages_dropped);
    }

    // 8. Graceful shutdown (drains queue, up to 5 s)
    lulu_shutdown();
    Ok(())
}
```

---

## 15. lulu-inject binary

The crate ships a ready-to-run demo injector:

```bash
# Default broker (127.0.0.1:1883)
cargo run --bin lulu-inject

# Custom broker
cargo run --bin lulu-inject -- 192.168.1.10 1883
```

What it does, in order:

1. `lulu_init` with the given broker.
2. Publishes ~60 realistic log entries (10 ms apart) across six virtual sources:
   - `psu/channel-1` and `psu/channel-2` — voltage (Float32), current, status, enabled, report.
   - `oscilloscope/probe-a` and `probe-b` — frequency (Float64), amplitude, clipped, sample-count, report.
   - `multimeter/dc` — voltage, resistance, continuity, range, overload, report.
   - `funcgen/output-1` — waveform, frequency, amplitude, offset, enabled.
   - `session/run-001` — name, step, result, summary.
   - `serial` — RX/TX `Bytes` payloads simulating a U-Boot → Linux boot sequence.
3. Injects three test scenarios:
   - `voltage-regulation-3v3` → success.
   - `overcurrent-protection` → failure with error message.
   - `signal-integrity-check` → begin only (in-progress / never ended).
4. Starts pulses for `psu/channel-1` (v1.0.0), `psu/channel-2` (v1.0.0), and `mcp/filesystem` (no version); waits 6 s; stops last two.
5. Prints stats and shuts down.

Use this binary as a reference implementation and to populate the UI with realistic data during development.

---

## 16. Constraints and limits

| Constraint | Value | Notes |
|---|---|---|
| Max payload | 20 480 bytes | `serializer::MAX_PAYLOAD_SIZE`; exceeded → message dropped with warning |
| QoS | 0 | No delivery guarantee; prefer higher volume / lower latency |
| Retain | false | Always |
| Timestamp format | `2026-03-11T10:30:00.123Z` | RFC 3339, ms precision, always UTC |
| Source characters | `[A-Za-z0-9\-_]` per segment | Uppercase allowed |
| Attribute | No `/`, non-empty | Any other printable ASCII is fine |
| Queue capacity | 256 (default) | Tune via `LuluClientConfig::queue_capacity` |
| Pulse interval | 2 s | Fixed; first pulse fires immediately |
| Offline threshold | 6 s | 3 × pulse interval (UI side) |
| Singleton | 1 per process | `OnceLock`; no reinit without restart |

---

## 17. Quick-reference cheat-sheet

```rust
// ── Setup ──────────────────────────────────────────────────────────────────
lulu_init(LuluClientConfig::default())?;               // must call first
lulu_init(LuluClientConfig { broker_host: "h".into(), broker_port: 1883, ..Default::default() })?;

// ── Publish ────────────────────────────────────────────────────────────────
lulu_publish(source, attribute, LogLevel::Info,  Data::String("msg".into()))?;
lulu_publish(source, attribute, LogLevel::Trace, Data::Float32(3.3))?;
lulu_publish(source, attribute, LogLevel::Warn,  Data::Float64(50.0))?;
lulu_publish(source, attribute, LogLevel::Debug, Data::Int32(42))?;
lulu_publish(source, attribute, LogLevel::Error, Data::Int64(1_000_000))?;
lulu_publish(source, attribute, LogLevel::Fatal, Data::Bool(false))?;
lulu_publish(source, attribute, LogLevel::Info,  Data::Json(r#"{"k":1}"#.into()))?;
lulu_publish(source, attribute, LogLevel::Info,  Data::Bytes(vec![0xAB]))?;

// ── Test scenarios ─────────────────────────────────────────────────────────
lulu_beg_test_scenario(source, attribute, "my-test")?;
lulu_end_test_scenario(source, attribute, "my-test", true, None)?;
lulu_end_test_scenario(source, attribute, "my-test", false, Some("reason"))?;

// ── Pulse ──────────────────────────────────────────────────────────────────
lulu_start_pulse(source, Some("1.0.0"))?;
lulu_start_pulse(source, None)?;
lulu_stop_pulse(source);

// ── Status ─────────────────────────────────────────────────────────────────
lulu_is_initialized()    // → bool
lulu_is_connected()      // → bool
lulu_stats()             // → Option<LuluStats>

// ── Teardown ───────────────────────────────────────────────────────────────
lulu_shutdown();         // drains queue (≤ 5 s), then returns
```
