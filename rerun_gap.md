# Rerun Gap Analysis: Mapping LuLu Logs Features to Rerun

This document maps **LuLu Logs** features and requirements to their **Rerun** equivalents, providing a clear migration path for implementing the same functionality using Rerun's data model and SDK.

---

## 📋 Table of Contents

1. [Core Logging Features](#1-core-logging-features)
2. [Hierarchical Keys and Entity Paths](#2-hierarchical-keys-and-entity-paths)
3. [Timestamps](#3-timestamps)
4. [Log Levels](#4-log-levels)
5. [Data Types](#5-data-types)
6. [Spans and Scenarios](#6-spans-and-scenarios)
7. [Streamable Format](#7-streamable-format)
8. [Binary Data Handling](#8-binary-data-handling)
9. [Query and Analysis](#9-query-and-analysis)
10. [Storage and File Format](#10-storage-and-file-format)
11. [Transport Protocols](#11-transport-protocols)
12. [Migration Checklist](#12-migration-checklist)

---

## 🎯 1. Core Logging Features

### LuLu Logs
- **Concept**: `LogRecord` is the fundamental unit with `key`, `timestamp_ns`, `level`, `type`, and `data`.
- **Schema**: Defined using FlatBuffers for binary serialization.
- **Use Case**: Unified logging for heterogeneous test data.

### Rerun Equivalent
- **Concept**: `rr.log(path, data)` is the fundamental logging operation.
- **Schema**: Uses Apache Arrow for columnar data storage.
- **Use Case**: Designed for multimodal data (scalars, images, point clouds, tensors, etc.).

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| `LogRecord` | `rr.log(path, data)` | Rerun's logging is entity-based with a hierarchical path. |
| FlatBuffers | Apache Arrow | Rerun uses Arrow for efficient storage and transport. |
| Binary format | `.rrd` files | Rerun's native file format is optimized for streaming and querying. |

### Code Example
**LuLu (Rust):**
```rust
LogRecord {
    key: "psu/channel-1/voltage",
    timestamp_ns: 1772044200123000000,
    level: LogLevel::Info,
    type: DataType::Float64,
    data: vec![0x40, 0x09, 0x1E, 0xB8, 0x51, 0xEB, 0x85, 0x1F], // 3.14 as Float64 LE
}
```

**Rerun (Python):**
```python
import rerun as rr

rr.init("lulu_migration")
rr.set_time_seconds("log_time", 1772044200.123)  # Convert ns to sec
rr.log("psu/channel-1/voltage", rr.Scalar(3.14))
```

**Rerun (Rust):**
```rust
use rerun::{RecordingStreamBuilder, datatypes::Scalar};

let rec = RecordingStreamBuilder::new("lulu_migration").connect_grpc()?;
let timestamp_sec = 1772044200123000000 as f64 / 1e9;
rec.set_time_seconds("log_time", timestamp_sec)?;
rec.log("psu/channel-1/voltage", &Scalar::new(3.14))?;
```

---

## 🏷️ 2. Hierarchical Keys and Entity Paths

### LuLu Logs
- **Format**: `{layer_1}/{layer_2}/.../{layer_n}` (e.g., `psu/power-supply/channel-1/voltage`).
- **Rules**: Alphanumeric segments, hyphen separator, max 256 characters.
- **Purpose**: Efficient routing and filtering of logs.

### Rerun Equivalent
- **Format**: Entity paths use the same hierarchical structure (e.g., `psu/power-supply/channel-1/voltage`).
- **Rules**: No strict length limit, but similar naming conventions apply.
- **Purpose**: Organizes entities in a tree structure for easy navigation in the viewer.

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| `key` | Entity `path` | Direct 1:1 mapping. |
| Hierarchical structure | Entity hierarchy | Rerun's viewer displays entities in a tree. |
| Routing/filtering | Path-based filtering | Use `WHERE path LIKE 'psu/%'` in DataFusion queries. |

### Code Example
**LuLu:**
```rust
key: "mcp/github/pull-request/status"
```

**Rerun:**
```python
rr.log("mcp/github/pull-request/status", rr.TextLog("PR opened"))
```

---

## ⏱️ 3. Timestamps

### LuLu Logs
- **Format**: `u64` nanoseconds since Unix epoch (1970-01-01T00:00:00Z).
- **Precision**: Nanoseconds.
- **Advantages**: Compact (8 bytes), fast to parse, no timezone ambiguity.

### Rerun Equivalent
- **Format**: `f64` seconds since Unix epoch (or other timelines like `frame_nr`).
- **Precision**: Sub-nanosecond (f64 precision).
- **Timelines**: Supports multiple timelines (e.g., `log_time`, `frame_nr`, `sim_time`).

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| `timestamp_ns` (u64) | `set_time_seconds(timeline, sec)` | Convert nanoseconds to seconds: `timestamp_sec = timestamp_ns / 1e9`. |
| Unix epoch | Unix epoch | Both use the same epoch. |
| UTC | UTC | Both assume UTC. |

### Code Example
**LuLu:**
```rust
timestamp_ns: 1772044200123000000  // 2026-02-26T14:30:00.123Z
```

**Rerun:**
```python
import time
timestamp_ns = 1772044200123000000
timestamp_sec = timestamp_ns / 1e9  # 1772044200.123
rr.set_time_seconds("log_time", timestamp_sec)
```

### Conversion Utilities
If you need to frequently convert between `u64` nanoseconds and Rerun's `f64` seconds:

**Python:**
```python
def ns_to_sec(ns: int) -> float:
    return ns / 1_000_000_000

def sec_to_ns(sec: float) -> int:
    return int(sec * 1_000_000_000)
```

**Rust:**
```rust
fn ns_to_sec(ns: u64) -> f64 {
    ns as f64 / 1_000_000_000.0
}

fn sec_to_ns(sec: f64) -> u64 {
    (sec * 1_000_000_000.0) as u64
}
```

---

## 📊 4. Log Levels

### LuLu Logs
- **Levels**: `Trace`, `Debug`, `Info`, `Warn`, `Error`, `Fatal` (enum `LogLevel`).
- **Default**: `Info`.
- **Purpose**: Indicate severity for filtering.

### Rerun Equivalent
- **Levels**: Rerun does not have built-in log levels, but you can:
  1. Use `class_name` to simulate levels (e.g., `class_name="Warn"`).
  2. Use `rr.TextLog` with a `level` parameter (if available in your Rerun version).
  3. Add a `level` component to your custom archetypes.

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| `level: LogLevel::Warn` | `class_name="Warn"` | Use `class_name` for filtering in the viewer. |
| `level: LogLevel::Error` | `class_name="Error"` | Consistent with Rerun's approach. |

### Code Example
**LuLu:**
```rust
level: LogLevel::Warn
```

**Rerun:**
```python
rr.log("psu/voltage", rr.Scalar(3.5), class_name="Warn")
```

### Custom Archetype for Log Levels (Advanced)
If you need strict typing for log levels, create a custom archetype:

**Rust:**
```rust
use rerun::{Archetype, AsComponents, datatypes::*};

#[derive(AsComponents, Archetype, Clone, Debug)]
#[archetype(name = "LogEntry")]
struct LogEntry {
    #[component]
    level: String,  // "Trace", "Debug", "Info", etc.
    
    #[component]
    message: String,
}

// Usage
let entry = LogEntry {
    level: "Warn".to_string(),
    message: "Voltage out of range".to_string(),
};
rec.log("psu/voltage", &entry)?;
```

---

## 📦 5. Data Types

### LuLu Logs
LuLu supports the following data types via the `DataType` enum:
- **Primitive types**: `String`, `Int32`, `Int64`, `Float32`, `Float64`, `Bool`, `Json`.
- **Binary types**: `Bytes`, `NetPacket`, `SerialChunk`.
- **Special types**: `SpanBeg`, `SpanEnd`, `ScenarioBeg`, `ScenarioEnd`, `StepBeg`, `StepEnd`.

### Rerun Equivalent
Rerun provides a rich set of **archetypes** for different data types:
- **Scalars**: `rr.Scalar` (supports `f32`, `f64`, `i32`, `i64`, `bool`, etc.).
- **Text**: `rr.TextLog` (for string messages).
- **JSON**: `rr.Json` (for structured data).
- **Binary**: Use `rr.TextLog` with base64/hex encoding or custom archetypes.
- **Time ranges**: `rr.TimeRange` (for spans/scenarios).
- **Multimodal**: `rr.Image`, `rr.Points3D`, `rr.Tensor`, etc.

### Implementation Mapping

#### Primitive Types
| LuLu `DataType` | Rerun Archetype | Example |
|-----------------|-----------------|---------|
| `String` | `rr.TextLog` | `rr.TextLog("Hello")` |
| `Int32` | `rr.Scalar` | `rr.Scalar(42)` |
| `Int64` | `rr.Scalar` | `rr.Scalar(42)` |
| `Float32` | `rr.Scalar` | `rr.Scalar(3.14)` |
| `Float64` | `rr.Scalar` | `rr.Scalar(3.14)` |
| `Bool` | `rr.Scalar` | `rr.Scalar(True)` |
| `Json` | `rr.Json` | `rr.Json({"key": "value"})` |

#### Binary Types
| LuLu `DataType` | Rerun Approach | Example |
|-----------------|----------------|---------|
| `Bytes` | `rr.TextLog` + base64 | `rr.TextLog(base64.b64encode(data).decode())` |
| `NetPacket` | Custom archetype or `rr.AnnotationContext` + `rr.TextLog` | See [Binary Data Handling](#8-binary-data-handling) |
| `SerialChunk` | Custom archetype or `rr.AnnotationContext` + `rr.TextLog` | See [Binary Data Handling](#8-binary-data-handling) |

#### Special Types (Spans/Scenarios)
| LuLu `DataType` | Rerun Archetype | Example |
|-----------------|-----------------|---------|
| `SpanBeg` | `rr.TimeRange` + `class_name="SpanBeg"` | See [Spans and Scenarios](#6-spans-and-scenarios) |
| `SpanEnd` | `rr.TimeRange` + `class_name="SpanEnd"` | See [Spans and Scenarios](#6-spans-and-scenarios) |
| `ScenarioBeg` | `rr.TimeRange` + `class_name="ScenarioBeg"` | See [Spans and Scenarios](#6-spans-and-scenarios) |
| `ScenarioEnd` | `rr.TimeRange` + `class_name="ScenarioEnd"` | See [Spans and Scenarios](#6-spans-and-scenarios) |
| `StepBeg` | `rr.TimeRange` + `class_name="StepBeg"` | See [Spans and Scenarios](#6-spans-and-scenarios) |
| `StepEnd` | `rr.TimeRange` + `class_name="StepEnd"` | See [Spans and Scenarios](#6-spans-and-scenarios) |

### Code Examples

#### Primitive Types
**LuLu:**
```rust
// Float64
LogRecord {
    key: "sensor/temperature",
    type: DataType::Float64,
    data: vec![0x40, 0x09, 0x1E, 0xB8, 0x51, 0xEB, 0x85, 0x1F], // 3.14
}

// String
LogRecord {
    key: "system/message",
    type: DataType::String,
    data: b"Hello, World!".to_vec(),
}

// Bool
LogRecord {
    key: "sensor/active",
    type: DataType::Bool,
    data: vec![0x01], // true
}
```

**Rerun:**
```python
# Float64
rr.log("sensor/temperature", rr.Scalar(3.14))

# String
rr.log("system/message", rr.TextLog("Hello, World!"))

# Bool
rr.log("sensor/active", rr.Scalar(True))
```

#### JSON
**LuLu:**
```rust
LogRecord {
    key: "event/data",
    type: DataType::Json,
    data: br#"{"key": "value", "count": 42}"#.to_vec(),
}
```

**Rerun:**
```python
rr.log("event/data", rr.Json({"key": "value", "count": 42}))
```

---

## 🔄 6. Spans and Scenarios

### LuLu Logs
LuLu uses special `DataType` values for spans and scenarios:
- **`SpanBeg`/`SpanEnd`**: Mark the start/end of a span (e.g., for tracing).
- **`ScenarioBeg`/`ScenarioEnd`**: Mark the start/end of a test scenario.
- **`StepBeg`/`StepEnd`**: Mark the start/end of a step within a scenario.
- **Data**: JSON-encoded metadata (e.g., `id`, `success`, `duration_ms`, `error`).

### Rerun Equivalent
Rerun uses **`TimeRange`** to represent spans/scenarios/steps:
- **`TimeRange`**: Defines a start and end time for an entity.
- **`class_name`**: Used to categorize entities (e.g., `"SpanBeg"`, `"ScenarioEnd"`).
- **Static components**: Used for metadata (e.g., `id`, `success`, `duration_ms`).

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| `SpanBeg` | `TimeRange(start, None)` + `class_name="SpanBeg"` | `end=None` indicates the span is ongoing. |
| `SpanEnd` | `TimeRange(start, end)` + `class_name="SpanEnd"` | Include metadata in `static` components. |
| `ScenarioBeg` | `TimeRange(start, None)` + `class_name="ScenarioBeg"` | Use `static` for metadata like `target_voltage`. |
| `ScenarioEnd` | `TimeRange(start, end)` + `class_name="ScenarioEnd"` | Include `success`, `duration_ms`, etc. in `static`. |
| `StepBeg` | `TimeRange(start, None)` + `class_name="StepBeg"` | Use `static` for step metadata. |
| `StepEnd` | `TimeRange(start, end)` + `class_name="StepEnd"` | Include results in `static`. |

### Code Examples

#### Basic Span
**LuLu:**
```rust
// SpanBeg
LogRecord {
    key: "spans/request-123",
    type: DataType::SpanBeg,
    data: br#"{"id": "request-123", "metadata": {"url": "/api/test"}}"#.to_vec(),
}

// SpanEnd
LogRecord {
    key: "spans/request-123",
    type: DataType::SpanEnd,
    data: br#"{"id": "request-123", "success": true, "duration_ms": 42}"#.to_vec(),
}
```

**Rerun:**
```python
import time

start_time = time.time()

# SpanBeg
rr.set_time_seconds("log_time", start_time)
rr.log(
    "spans/request-123",
    rr.TimeRange(start=start_time, end=None),
    class_name="SpanBeg",
    static={"id": "request-123", "url": "/api/test"}
)

# SpanEnd
end_time = time.time()
rr.set_time_seconds("log_time", end_time)
rr.log(
    "spans/request-123",
    rr.TimeRange(start=start_time, end=end_time),
    class_name="SpanEnd",
    static={"id": "request-123", "success": True, "duration_ms": (end_time - start_time) * 1000}
)
```

#### Scenario with Steps
**LuLu:**
```rust
// ScenarioBeg
LogRecord {
    key: "test/scenario/voltage-regulation-3v3",
    type: DataType::ScenarioBeg,
    data: br#"{"id": "voltage-regulation-3v3", "metadata": {"target_voltage": 3.3, "tolerance_v": 0.05}}"#.to_vec(),
}

// StepBeg
LogRecord {
    key: "test/scenario/voltage-regulation-3v3/measure-voltage",
    type: DataType::StepBeg,
    data: br#"{"id": "measure-voltage", "metadata": {"channel": 1}}"#.to_vec(),
}

// StepEnd
LogRecord {
    key: "test/scenario/voltage-regulation-3v3/measure-voltage",
    type: DataType::StepEnd,
    data: br#"{"id": "measure-voltage", "success": true, "duration_ms": 5, "result": {"measured_v": 3.31}}"#.to_vec(),
}

// ScenarioEnd
LogRecord {
    key: "test/scenario/voltage-regulation-3v3",
    type: DataType::ScenarioEnd,
    data: br#"{"id": "voltage-regulation-3v3", "success": true, "duration_ms": 24, "result": {"measured_min": 3.30, "measured_max": 3.31}}"#.to_vec(),
}
```

**Rerun:**
```python
import time

start_time = time.time()
step_start = start_time + 0.1
measure_time = step_start + 0.005
step_end = step_start + 0.01
scenario_end = start_time + 0.2

# ScenarioBeg
rr.set_time_seconds("log_time", start_time)
rr.log(
    "test/scenario/voltage-regulation-3v3",
    rr.TimeRange(start=start_time, end=None),
    class_name="ScenarioBeg",
    static={
        "id": "voltage-regulation-3v3",
        "target_voltage": 3.3,
        "tolerance_v": 0.05
    }
)

# StepBeg
rr.set_time_seconds("log_time", step_start)
rr.log(
    "test/scenario/voltage-regulation-3v3/measure-voltage",
    rr.TimeRange(start=step_start, end=None),
    class_name="StepBeg",
    static={"id": "measure-voltage", "channel": 1}
)

# Log a measurement (Float64)
rr.set_time_seconds("log_time", measure_time)
rr.log(
    "test/scenario/voltage-regulation-3v3/measure-voltage/voltage",
    rr.Scalar(3.31)
)

# StepEnd
rr.set_time_seconds("log_time", step_end)
rr.log(
    "test/scenario/voltage-regulation-3v3/measure-voltage",
    rr.TimeRange(start=step_start, end=step_end),
    class_name="StepEnd",
    static={
        "id": "measure-voltage",
        "success": True,
        "duration_ms": (step_end - step_start) * 1000,
        "measured_v": 3.31
    }
)

# ScenarioEnd
rr.set_time_seconds("log_time", scenario_end)
rr.log(
    "test/scenario/voltage-regulation-3v3",
    rr.TimeRange(start=start_time, end=scenario_end),
    class_name="ScenarioEnd",
    static={
        "id": "voltage-regulation-3v3",
        "success": True,
        "duration_ms": (scenario_end - start_time) * 1000,
        "measured_min": 3.30,
        "measured_max": 3.31
    }
)
```

### Viewer Integration
- **TimeRanges** appear as **horizontal bars** in the Rerun viewer's timeline.
- **Filter by `class_name`** to show only spans, scenarios, or steps.
- **Click on a `TimeRange`** to inspect its metadata (e.g., `id`, `success`, `duration_ms`).

---

## 🚀 7. Streamable Format

### LuLu Logs
- **Format**: `[u32 length (big-endian)][FlatBuffer LogRecord]...`
- **Advantages**:
  - Sequential reading without loading the entire file.
  - Append-only (O(1) for adding records).
  - Memory-efficient (read record by record).
  - Corruption-resistant (one bad record doesn't break the entire stream).
- **Transport**: Works with MQTT, TCP, files, etc.

### Rerun Equivalent
- **Format**: `.rrd` files (Apache Arrow-based) or gRPC streaming.
- **Advantages**:
  - **Append-only**: New data is appended to the end of the file.
  - **Corruption-resistant**: Each chunk is independent.
  - **Compression**: Native support for Arrow compression.
  - **Multi-transport**: gRPC, files, WebSocket (via viewer).
- **Streaming**: Use `RecordingStream` for real-time logging.

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| Length-prefixed records | `.rrd` file format | Rerun's `.rrd` files are append-only and corruption-resistant. |
| FlatBuffers | Apache Arrow | Rerun uses Arrow for efficient storage and transport. |
| Big-endian `u32` length | Arrow metadata | Arrow handles chunk metadata internally. |
| MQTT transport | gRPC or custom bridge | Rerun natively supports gRPC; use a bridge for MQTT. |

### Code Examples

#### Writing to a File
**LuLu:**
```rust
// Pseudocode for LuLu's file writing
let mut file = File::create("logs.lulu")?;
for record in log_records {
    let flatbuffer = serialize_to_flatbuffer(&record);
    let length = flatbuffer.len() as u32;
    file.write_all(&length.to_be_bytes())?;
    file.write_all(&flatbuffer)?;
}
```

**Rerun:**
```python
import rerun as rr

rr.init("lulu_migration")
# Log some data...
rr.log("psu/voltage", rr.Scalar(3.14))
# Save to file
rr.save("logs.rrd")  # Append-only, corruption-resistant
```

#### Streaming Over gRPC
**LuLu:**
```rust
// Pseudocode for LuLu's MQTT/TCP streaming
let mut stream = TcpStream::connect("log-server:1883")?;
for record in log_records {
    let flatbuffer = serialize_to_flatbuffer(&record);
    let length = flatbuffer.len() as u32;
    stream.write_all(&length.to_be_bytes())?;
    stream.write_all(&flatbuffer)?;
}
```

**Rerun:**
```python
import rerun as rr

# Connect to a gRPC server
rec = rr.connect("localhost:9876")
# Log data in real-time
rec.log("psu/voltage", rr.Scalar(3.14))
```

#### MQTT Bridge (If Needed)
If you must keep MQTT, create a bridge to Rerun:

**Python:**
```python
import paho.mqtt.client as mqtt
import rerun as rr

def on_message(client, userdata, msg):
    # Parse LuLu's FlatBuffer message
    log_record = parse_lulu_log(msg.payload)
    
    # Convert to Rerun
    timestamp_sec = log_record.timestamp_ns / 1e9
    rr.set_time_seconds("log_time", timestamp_sec)
    
    # Map data type to Rerun archetype
    if log_record.type == DataType.Float64:
        value = struct.unpack('<d', log_record.data)[0]
        rr.log(log_record.key, rr.Scalar(value))
    elif log_record.type == DataType.String:
        text = log_record.data.decode('utf-8')
        rr.log(log_record.key, rr.TextLog(text))
    # ... handle other types

client = mqtt.Client()
client.on_message = on_message
client.connect("mqtt_broker", 1883)
client.subscribe("lulu/#")
client.loop_forever()
```

---

## 🗃️ 8. Binary Data Handling

### LuLu Logs
LuLu supports binary data via:
- **`Bytes`**: Opaque binary data.
- **`NetPacket`**: Network packets.
- **`SerialChunk`**: Serial communication chunks.

### Rerun Equivalent
Rerun does not have native support for arbitrary binary data, but you can:
1. **Encode as base64/hex** and use `TextLog`.
2. **Create a custom archetype** for structured binary data (e.g., `NetPacket`).
3. **Use `AnnotationContext`** to add metadata to binary blobs.

### Implementation Mapping
| LuLu Feature | Rerun Approach | Notes |
|--------------|----------------|-------|
| `Bytes` | `TextLog` + base64 | Simple and effective for small binary data. |
| `NetPacket` | Custom archetype | Best for structured network data. |
| `SerialChunk` | Custom archetype | Best for structured serial data. |

### Code Examples

#### Base64 Encoding (Simple)
**LuLu:**
```rust
LogRecord {
    key: "sensor/raw_data",
    type: DataType::Bytes,
    data: vec![0x01, 0x02, 0x03, 0x04],
}
```

**Rerun:**
```python
import base64

data = b'\x01\x02\x03\x04'
rr.log("sensor/raw_data", rr.TextLog(base64.b64encode(data).decode()))
```

#### Custom Archetype for NetPacket (Advanced)
**Rust:**
```rust
use rerun::{Archetype, AsComponents, datatypes::*};

#[derive(AsComponents, Archetype, Clone, Debug)]
#[archetype(name = "NetPacket")]
struct NetPacket {
    #[component]
    src_ip: String,
    
    #[component]
    dst_ip: String,
    
    #[component]
    protocol: String,  // "TCP", "UDP", etc.
    
    #[component]
    payload: Vec<u8>,  // Raw payload (Rerun will handle serialization)
}

// Usage
let packet = NetPacket {
    src_ip: "192.168.1.1".to_string(),
    dst_ip: "192.168.1.2".to_string(),
    protocol: "TCP".to_string(),
    payload: vec![0x01, 0x02, 0x03, 0x04],
};
rec.log("network/packets", &packet)?;
```

**Python (using `rerun` SDK):**
```python
# Rerun does not yet support custom archetypes in Python,
# but you can use a combination of components:
rr.log(
    "network/packets",
    rr.AnnotationContext(
        label="NetPacket",
        description="TCP packet from 192.168.1.1 to 192.168.1.2"
    ),
    rr.TextLog(base64.b64encode(b'\x01\x02\x03\x04').decode())
)
```

#### Using AnnotationContext for Metadata
**Rerun:**
```python
import base64

data = b'\x01\x02\x03\x04'
rr.log(
    "sensor/raw_data",
    rr.AnnotationContext(
        label="Raw Bytes",
        description=f"Size: {len(data)} bytes, Type: SerialChunk"
    ),
    rr.TextLog(base64.b64encode(data).decode())
)
```

---

## 🔍 9. Query and Analysis

### LuLu Logs
- **Current State**: No built-in query engine. Logs are stored in binary files.
- **Future Need**: Ability to filter, aggregate, and analyze logs.

### Rerun Equivalent
Rerun uses **DataFusion** (a query engine built on Apache Arrow) to:
- **Filter** logs by path, time, or metadata.
- **Aggregate** data (e.g., average, max, count).
- **Export** to Pandas, Arrow, or Parquet.
- **Query** using SQL-like syntax.

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| Filter by `key` | `WHERE path LIKE 'psu/%'` | Use DataFusion's SQL syntax. |
| Filter by `timestamp` | `WHERE log_time > 1772044200.0` | Filter by time range. |
| Aggregate (avg, max) | `SELECT path, AVG(value) FROM /sensors/*` | Use SQL aggregation. |
| Export to CSV/Pandas | `.to_pandas()` | Convert query results to Pandas. |

### Code Examples

#### Basic Query
**Rerun:**
```python
from rerun import Server, Client

# Start a server with your .rrd files
with Server(datasets={"lulu": ["logs.rrd"]}) as server:
    client = Client(server.url())
    dataset = client.get_dataset("lulu")
    
    # Query all voltage logs
    df = dataset.query("SELECT * FROM /psu/*")
    print(df.to_pandas())
```

#### Filter by Time
**Rerun:**
```python
# Filter logs between two timestamps
df = dataset.query(
    "SELECT * FROM /test/* WHERE log_time > 1772044200.0 AND log_time < 1772044201.0"
)
```

#### Aggregate Data
**Rerun:**
```python
# Calculate average voltage per channel
df = dataset.query(
    "SELECT path, AVG(value) as avg_voltage FROM /psu/* GROUP BY path"
)
```

#### Filter by Class Name (Spans/Scenarios)
**Rerun:**
```python
# Get all spans
df = dataset.query("SELECT * FROM /* WHERE class_name = 'SpanBeg' OR class_name = 'SpanEnd'")
```

#### Export to Pandas
**Rerun:**
```python
# Export query results to Pandas
pandas_df = df.to_pandas()
print(pandas_df.head())
```

---

## 💾 10. Storage and File Format

### LuLu Logs
- **Format**: Binary files with length-prefixed FlatBuffer records.
- **Structure**: `[u32 length][FlatBuffer LogRecord]...`
- **Advantages**:
  - Append-only.
  - Corruption-resistant.
  - Memory-efficient for sequential reading.

### Rerun Equivalent
- **Format**: `.rrd` files (Rerun Recording Data).
- **Structure**: Apache Arrow-based with metadata.
- **Advantages**:
  - **Append-only**: New data is appended to the end.
  - **Corruption-resistant**: Each chunk is independent.
  - **Compression**: Native Arrow compression (e.g., Zstd, LZ4).
  - **Metadata**: Stores schema, timelines, and other metadata.
  - **Backward compatibility**: Rerun guarantees backward compatibility for `.rrd` files (since v0.23).

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| Binary files | `.rrd` files | Rerun's native format. |
| Append-only | Append-only | Both support appending new records. |
| Corruption-resistant | Corruption-resistant | Both handle corruption gracefully. |
| No compression | Compression (Arrow) | Rerun compresses data by default. |

### Code Examples

#### Saving to a File
**LuLu:**
```rust
// Pseudocode: Write LogRecords to a file
let mut file = File::create("logs.lulu")?;
for record in records {
    let flatbuffer = serialize(&record);
    let length = flatbuffer.len() as u32;
    file.write_all(&length.to_be_bytes())?;
    file.write_all(&flatbuffer)?;
}
```

**Rerun:**
```python
import rerun as rr

rr.init("lulu_migration")
# Log some data...
rr.log("psu/voltage", rr.Scalar(3.14))
# Save to .rrd file
rr.save("logs.rrd")
```

#### Loading from a File
**LuLu:**
```rust
// Pseudocode: Read LogRecords from a file
let mut file = File::open("logs.lulu")?;
while let Some(record) = read_next_record(&mut file) {
    println!("Key: {}, Timestamp: {}", record.key, record.timestamp_ns);
}
```

**Rerun:**
```python
import rerun as rr

# Load a .rrd file in the viewer
rr.view("logs.rrd")
```

#### Migration Script (LuLu → Rerun)
**Python:**
```python
import rerun as rr
import struct

def migrate_lulu_to_rerun(lulu_file_path, rrd_file_path):
    """
    Migrate a LuLu Logs binary file to a Rerun .rrd file.
    
    Args:
        lulu_file_path: Path to the LuLu Logs binary file.
        rrd_file_path: Path to save the Rerun .rrd file.
    """
    rr.init("lulu_migration")
    
    with open(lulu_file_path, "rb") as f:
        while True:
            # Read length (u32 big-endian)
            length_bytes = f.read(4)
            if len(length_bytes) != 4:
                break  # EOF
            length = struct.unpack(">I", length_bytes)[0]
            
            # Read FlatBuffer data
            flatbuffer_data = f.read(length)
            if len(flatbuffer_data) != length:
                break  # Corrupted record
            
            # Parse FlatBuffer (pseudocode - use actual FlatBuffers parsing)
            log_record = parse_flatbuffer(flatbuffer_data)
            
            # Convert to Rerun
            timestamp_sec = log_record.timestamp_ns / 1e9
            rr.set_time_seconds("log_time", timestamp_sec)
            
            # Map data type to Rerun archetype
            if log_record.type == DataType.Float64:
                value = struct.unpack('<d', log_record.data)[0]
                rr.log(log_record.key, rr.Scalar(value))
            elif log_record.type == DataType.String:
                text = log_record.data.decode('utf-8')
                rr.log(log_record.key, rr.TextLog(text))
            elif log_record.type == DataType.SpanBeg:
                metadata = json.loads(log_record.data.decode('utf-8'))
                rr.log(
                    log_record.key,
                    rr.TimeRange(start=timestamp_sec, end=None),
                    class_name="SpanBeg",
                    static=metadata
                )
            # ... handle other types
    
    # Save to .rrd file
    rr.save(rrd_file_path)

# Usage
migrate_lulu_to_rerun("logs.lulu", "logs.rrd")
```

---

## 🌐 11. Transport Protocols

### LuLu Logs
- **Supported**: MQTT, TCP, Files.
- **MQTT**: Uses topics for routing (e.g., `lulu/psu/voltage`).
- **TCP**: Custom framing with length-prefixed records.
- **Files**: Binary files with length-prefixed records.

### Rerun Equivalent
- **Supported**: gRPC, Files (`.rrd`), WebSocket (via viewer).
- **gRPC**: Native support for real-time streaming.
- **Files**: `.rrd` files for persistent storage.
- **WebSocket**: Supported via the Rerun viewer.

### Implementation Mapping
| LuLu Feature | Rerun Equivalent | Notes |
|--------------|------------------|-------|
| MQTT | gRPC or custom bridge | Rerun does not natively support MQTT; use a bridge. |
| TCP | gRPC | gRPC is built on HTTP/2 and is more efficient than raw TCP. |
| Files | `.rrd` files | Rerun's native file format. |

### Code Examples

#### gRPC Streaming
**Rerun:**
```python
import rerun as rr

# Connect to a gRPC server
rec = rr.connect("localhost:9876")

# Log data in real-time
rec.log("psu/voltage", rr.Scalar(3.14))
```

#### File-Based Logging
**Rerun:**
```python
import rerun as rr

# Save to a file
rr.init("lulu_migration")
rr.log("psu/voltage", rr.Scalar(3.14))
rr.save("logs.rrd")
```

#### MQTT Bridge to Rerun
If you must keep MQTT, create a bridge:

**Python:**
```python
import paho.mqtt.client as mqtt
import rerun as rr

# Initialize Rerun
rr.init("mqtt_bridge")

def on_connect(client, userdata, flags, rc):
    client.subscribe("lulu/#")

def on_message(client, userdata, msg):
    # Parse LuLu's message (FlatBuffer)
    log_record = parse_lulu_log(msg.payload)
    
    # Convert to Rerun
    timestamp_sec = log_record.timestamp_ns / 1e9
    rr.set_time_seconds("log_time", timestamp_sec)
    
    # Map data type to Rerun archetype
    if log_record.type == DataType.Float64:
        value = struct.unpack('<d', log_record.data)[0]
        rr.log(log_record.key, rr.Scalar(value))
    elif log_record.type == DataType.String:
        text = log_record.data.decode('utf-8')
        rr.log(log_record.key, rr.TextLog(text))
    # ... handle other types

client = mqtt.Client()
client.on_connect = on_connect
client.on_message = on_message
client.connect("mqtt_broker", 1883)
client.loop_forever()
```

---

## ✅ 12. Migration Checklist

Use this checklist to track your migration from LuLu Logs to Rerun.

### Phase 1: Setup and Prototyping
- [ ] Install Rerun SDK (`pip install rerun-sdk` or `cargo add rerun`).
- [ ] Test basic logging with `rr.log()`.
- [ ] Verify the Rerun viewer works (`rerun` or `rr.spawn()`).
- [ ] Benchmark Rerun's performance vs. LuLu (write/read 10k records).

### Phase 2: Core Logging Migration
- [ ] Replace `LogRecord` with `rr.log(path, data)`.
- [ ] Map LuLu's `key` to Rerun's entity `path`.
- [ ] Convert `timestamp_ns` (u64) to `f64` seconds for Rerun.
- [ ] Map LuLu's `level` to Rerun's `class_name` or custom archetype.
- [ ] Map primitive data types (`String`, `Int32`, `Float64`, etc.) to Rerun archetypes (`Scalar`, `TextLog`, etc.).

### Phase 3: Advanced Features
- [ ] Implement spans/scenarios using `TimeRange` + `class_name`.
- [ ] Handle binary data (`Bytes`, `NetPacket`, `SerialChunk`) using base64 encoding or custom archetypes.
- [ ] Test JSON data with `rr.Json`.
- [ ] Verify metadata (e.g., `id`, `success`, `duration_ms`) is preserved in `static` components.

### Phase 4: Transport and Storage
- [ ] Replace MQTT/TCP with Rerun's gRPC or `.rrd` files.
- [ ] If MQTT is required, implement a bridge to Rerun.
- [ ] Migrate existing LuLu binary files to `.rrd` using a migration script.
- [ ] Test append-only behavior with `.rrd` files.

### Phase 5: Query and Analysis
- [ ] Set up a Rerun server for querying `.rrd` files.
- [ ] Test DataFusion queries (filter by path, time, `class_name`).
- [ ] Export query results to Pandas/Arrow.
- [ ] Document common queries for your use cases.

### Phase 6: Viewer Integration
- [ ] Configure the Rerun viewer for your data.
- [ ] Set up timelines (`log_time`, `frame_nr`, etc.).
- [ ] Test filtering by `class_name` (e.g., show only spans).
- [ ] Train your team on using the Rerun viewer.

### Phase 7: Optimization (Optional)
- [ ] Create custom archetypes for domain-specific data (e.g., `NetPacket`).
- [ ] Integrate with OpenTelemetry for text logs/metrics.
- [ ] Optimize for embedded systems (buffer size, compression).
- [ ] Benchmark and tune performance.

---

## 📌 Summary Table: LuLu Logs → Rerun Mapping

| **LuLu Feature**               | **Rerun Equivalent**                          | **Complexity** | **Notes** |
|--------------------------------|---------------------------------------------|----------------|-----------|
| `LogRecord`                    | `rr.log(path, data)`                        | Low            | Direct mapping. |
| `key` (hierarchical)           | Entity `path`                               | Low            | Same concept. |
| `timestamp_ns` (u64)           | `set_time_seconds(timeline, sec)`          | Low            | Convert ns → sec. |
| `level` (enum)                 | `class_name` or custom archetype            | Low            | Use `class_name="Warn"`. |
| `String`                       | `rr.TextLog`                                | Low            | Direct mapping. |
| `Int32`/`Int64`/`Float32`/`Float64` | `rr.Scalar`                          | Low            | Direct mapping. |
| `Bool`                         | `rr.Scalar`                                | Low            | Direct mapping. |
| `Json`                         | `rr.Json`                                  | Low            | Direct mapping. |
| `Bytes`                        | `rr.TextLog` + base64                      | Medium          | Encode binary data. |
| `NetPacket`/`SerialChunk`      | Custom archetype or `AnnotationContext` + `TextLog` | High | Requires custom code. |
| `SpanBeg`/`SpanEnd`            | `rr.TimeRange` + `class_name`              | Medium          | Use `static` for metadata. |
| `ScenarioBeg`/`ScenarioEnd`    | `rr.TimeRange` + `class_name`              | Medium          | Use `static` for metadata. |
| `StepBeg`/`StepEnd`            | `rr.TimeRange` + `class_name`              | Medium          | Use `static` for metadata. |
| Length-prefixed binary format | `.rrd` files (Arrow-based)                  | Low            | Rerun handles this internally. |
| MQTT transport                 | gRPC or custom bridge                      | Medium          | Use a bridge if MQTT is required. |
| TCP transport                  | gRPC                                      | Low            | gRPC is built on HTTP/2. |
| File storage                   | `.rrd` files                               | Low            | Direct replacement. |
| No query engine                | DataFusion (SQL)                           | Low            | Built-in support. |
| No viewer                      | Rerun Viewer (native/Web)                  | Low            | Built-in support. |

---

## 🚀 Next Steps

1. **Start Small**: Migrate a single use case (e.g., scalar logging) to Rerun.
2. **Test the Viewer**: Verify that your data is displayed correctly.
3. **Iterate**: Gradually migrate more features (spans, scenarios, binary data).
4. **Document**: Update your documentation to reflect the new Rerun-based approach.
5. **Train**: Ensure your team is familiar with Rerun's viewer and query engine.

---

## 📚 References
- [Rerun Documentation](https://docs.rerun.io)
- [Rerun GitHub](https://github.com/rerun-io/rerun)
- [Rerun Python SDK](https://docs.rerun.io/latest/python)
- [Rerun Rust SDK](https://docs.rs/rerun)
- [DataFusion (Query Engine)](https://docs.rerun.io/latest/howto/get-data-out)
- [Apache Arrow](https://arrow.apache.org/)
