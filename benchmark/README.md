# Lulu-Logs Serialization Benchmark

This directory contains a benchmark comparing Protobuf Length-Delimited vs FlatBuffers Multiples for the lulu-logs serialization format.

## Purpose

The current lulu-logs implementation uses FlatBuffers with a single buffer for all records, which requires reading the entire file to add a new record (O(n) complexity).

This benchmark tests two append-only alternatives:
1. Protobuf Length-Delimited: Each record is a Protobuf message prefixed with its length (varint)
2. FlatBuffers Multiples: Each record is a separate FlatBuffer prefixed with its length (u32)

## Structure

benchmark/
- Cargo.toml - Benchmark project configuration
- build.rs - Protobuf code generation
- protobuf/lulu_logs.proto - Protobuf schema
- flatbuffers/lulu_logs.fbs - FlatBuffers schema
- benches/serialization_bench.rs - Benchmark implementations

## Setup

### Prerequisites
- Rust (latest stable version)
- Cargo (Rust package manager)
- FlatBuffers compiler (flatc) - Optional, only needed if you modify the schema

### Install FlatBuffers compiler (optional)
On Ubuntu: sudo apt-get install flatbuffers-compiler
On macOS: brew install flatbuffers
Or download from https://github.com/google/flatbuffers/releases

### Build
cd benchmark
cargo build --release

## Running Benchmarks

cargo bench

### Benchmark Groups
- write_performance: Measures write speed for 100, 1000, 10000 records
- read_sequential: Measures sequential read speed
- random_access: Measures time to read the 5000th record
- file_size: Compares file sizes for 1000, 10000 records

## Expected Results

Protobuf LD is expected to be:
- 20-30% faster for writes
- 20% smaller in file size
- Slightly slower for random access (but negligible for sequential reads)

FlatBuffers Multiples is expected to be:
- Slightly faster for random access
- 20% larger in file size
- 20-30% slower for writes

## Implementation Details

### Protobuf Length-Delimited
Format: [varint length][protobuf data]
Schema: message LogRecord { string topic = 1; bytes payload = 2; uint64 timestamp_ns = 3; }

### FlatBuffers Multiples
Format: [u32 length][flatbuffer data]
Schema: table LogRecord { topic: string; payload: [ubyte]; timestamp_ns: ulong; }

## Recommendations

For lulu-logs, we recommend Protobuf Length-Delimited because:
1. Smaller file size
2. Faster writes
3. Simpler implementation
4. Better tooling
5. Better backward compatibility

Use FlatBuffers Multiples only if you need zero-copy access to individual fields.

## Notes

Both formats support:
- Append-only writes (O(1) per record)
- Robustness (corrupted record does not affect others)
- Multi-language support (Rust, Python, C++, etc.)
- Realistic data (100-byte payloads, MQTT-style topics)
