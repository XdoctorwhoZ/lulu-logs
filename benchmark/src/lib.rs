//! Benchmark module for comparing Protobuf vs FlatBuffers serialization
//! for lulu-logs

pub mod protobuf;
pub mod flatbuffers;
pub mod lulu_logs_generated;

// Re-export for benchmarks
pub use protobuf::{ProtobufLogWriter, ProtobufLogReader};
pub use flatbuffers::{FlatBuffersLogWriter, FlatBuffersLogReader};
