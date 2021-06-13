use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let proto_files = vec![
        root.join("handler_types.proto"),
        root.join("handler_service.proto"),
    ];

    // Tell cargo to recompile if any of these proto files are changed
    for proto_file in &proto_files {
        println!("cargo:rerun-if-changed={}", proto_file.display());
    }

    tonic_build::configure().compile(&proto_files, &[root]).unwrap();
}