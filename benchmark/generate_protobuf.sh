#!/bin/bash
# Generate Protobuf Rust code
# Requires: protoc, prost, protobuf

echo "Generating Protobuf code..."
cd $(dirname "$0")

# Generate Rust code from .proto file
protoc --rust-out . --proto-path protobuf protobuf/lulu_logs.proto

# Rename generated file to match module
mv protobuf/lulu_logs.rs src/ 2>/dev/null || true

echo "Protobuf code generated in src/lulu_logs.rs"
