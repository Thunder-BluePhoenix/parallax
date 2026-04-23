fn main() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile_protos(&["../proto/parallax.proto"], &["../proto"])
        .expect("Failed to compile protos");

    tauri_build::build()
}
