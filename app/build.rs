use std::process::Command;

fn main() {
    // Trigger rebuild when schemas change
    println!("cargo:rerun-if-changed=../schema/lulu_logs.fbs");
    println!("cargo:rerun-if-changed=../schema/lulu_export.fbs");

    // Generate Rust bindings for lulu_logs.fbs
    let status = Command::new("flatc")
        .args(["--rust", "-o", "src/generated/", "../schema/lulu_logs.fbs"])
        .status()
        .expect("flatc not found — install FlatBuffers compiler (brew install flatbuffers)");
    assert!(status.success(), "flatc failed on lulu_logs.fbs");

    // Generate Rust bindings for lulu_export.fbs
    let status = Command::new("flatc")
        .args(["--rust", "-o", "src/generated/", "../schema/lulu_export.fbs"])
        .status()
        .expect("flatc not found — install FlatBuffers compiler (brew install flatbuffers)");
    assert!(status.success(), "flatc failed on lulu_export.fbs");
}
