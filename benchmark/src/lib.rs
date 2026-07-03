//! Benchmark module for comparing Protobuf vs FlatBuffers serialization
//! for lulu-logs

pub mod protobuf;
pub mod flatbuffers;

// Re-export for benchmarks
pub use protobuf::{ProtobufLogWriter, ProtobufLogReader};
pub use flatbuffers::{FlatBuffersLogWriter, FlatBuffersLogReader};
