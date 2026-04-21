fn main() {
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(&["../proto/parallax.proto"], &["../proto"])
        .unwrap_or_else(|e| panic!("Failed to compile protos {:?}", e));

    tauri_build::build()
}
