# LULU LOGS

A desktop application and Rust client library for viewing and managing logs over MQTT using the lulu-logs protocol.

## Overview

**lulu-logs** is a comprehensive logging solution built for the Panduza project that provides:

- **Desktop Log Viewer**: A cross-platform desktop application built with Dioxus that connects to MQTT brokers and displays real-time logs
- **Rust Client Library**: A reusable Rust client for programmatic access to the lulu-logs protocol
- **MQTT Protocol**: A structured logging protocol using MQTT topics and FlatBuffers for efficient serialization
- **Test Scenario Support**: Capability to inject test scenarios and validate log outputs

## Project Structure

```
.
├── app/                          # Desktop application (Dioxus)
│   ├── src/
│   │   ├── components/          # UI components (toolbar, log list, status bar, etc.)
│   │   ├── models/              # Data models and domain logic
│   │   ├── mqtt/                # MQTT broker and client functionality
│   │   ├── generated/           # FlatBuffers generated code
│   │   └── main.rs / app.rs     # Application entry point
│   └── assets/                  # Static assets (CSS, etc.)
├── rust-client/                 # Rust client library
│   ├── src/
│   │   ├── client.rs            # MQTT client implementation
│   │   ├── models.rs            # Data models
│   │   ├── serializer.rs        # FlatBuffers serialization
│   │   ├── topic.rs             # Topic parsing and construction
│   │   └── bin/inject.rs        # CLI tool for injecting test logs
│   └── schema/                  # FlatBuffers schema definitions
├── schema/                      # Shared FlatBuffers schemas
│   ├── lulu_logs.fbs           # Main protocol schema
│   └── lulu_export.fbs         # Export schema
├── specs/                       # Protocol specification documents
└── Cargo.toml                   # Workspace configuration

```

## Prerequisites

- Rust 1.70+ (2021 edition)
- An MQTT broker (local or remote)
- macOS, Linux, or Windows

## Building

### Desktop Application

```bash
cd app
cargo build --release
```

The built application will be in `target/aarch64-apple-darwin/desktop-dev/` (or appropriate target directory).

### Rust Client Library

```bash
cd rust-client
cargo build --release
```

### Test Injection Tool

```bash
cd rust-client
cargo build --release --bin inject
```

## Running

### Desktop Application

```bash
cd app
cargo run
```

The application will start a UI to configure MQTT connection settings and display incoming logs in real-time.

### Injecting Test Logs

Use the provided CLI tool to inject test logs into the system:

```bash
cargo run --release --bin inject -- --broker <BROKER_ADDRESS> --topic <TOPIC> --message <MESSAGE>
```

## Protocol

The lulu-logs protocol defines a structured MQTT-based logging format:

- **Topics**: Follow the pattern `lulu/{source_segments...}/{attribute_name}`
- **Payloads**: Binary FlatBuffers-encoded data for efficient serialization
- **Features**: Support for various data types, timestamps, and test scenarios

For detailed protocol specification, see [specs/lulu-logs.md](specs/lulu-logs.md).

## Dependencies

- **dioxus**: Reactive UI framework for the desktop application
- **rumqttc**: MQTT client library
- **flatbuffers**: Binary serialization format
- **tokio**: Async runtime
- **serde**: Serialization framework
- **chrono**: Date and time handling
- **tracing**: Structured logging

## Development

### Code Generation

FlatBuffers schemas are automatically compiled to Rust code during the build process. Schema files are located in:
- `schema/` - Shared schemas
- `rust-client/schema/` - Client-specific schemas

### Testing

Run tests with:

```bash
cargo test
```

## Features

- ✅ Real-time log viewing from MQTT streams
- ✅ Multiple source filtering and display
- ✅ Configurable MQTT connection
- ✅ Test scenario injection
- ✅ Type-safe FlatBuffers serialization
- ✅ Cross-platform desktop application

## License

Part of the Panduza project.


