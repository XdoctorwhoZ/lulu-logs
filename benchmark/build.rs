fn main() {
    // Generate Protobuf code
    prost_build::compile_protos(
        &["protobuf/lulu_logs.proto"],
        &["protobuf/"],
    ).unwrap();

    // Note: FlatBuffers code is generated manually with flatc
    // To generate: flatc --rust flatbuffers/lulu_logs.fbs -o src/
    println!("cargo:rerun-if-changed=protobuf/lulu_logs.proto");
    println!("cargo:rerun-if-changed=flatbuffers/lulu_logs.fbs");
}
