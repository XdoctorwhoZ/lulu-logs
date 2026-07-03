fn main() {
    // Generate Protobuf code
    prost_build::compile_protos(
        &["benchmark/protobuf/lulu_logs.proto"],
        &["benchmark/protobuf/"],
    ).unwrap();

    // Note: FlatBuffers code is generated manually with flatc
    // To generate: flatc --rust benchmark/flatbuffers/lulu_logs.fbs -o benchmark/src/
    println!("cargo:rerun-if-changed=benchmark/protobuf/lulu_logs.proto");
    println!("cargo:rerun-if-changed=benchmark/flatbuffers/lulu_logs.fbs");
}
