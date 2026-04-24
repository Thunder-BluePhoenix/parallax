fn main() {
    // Compile protobuf definitions for the Go sidecar gRPC client.
    // In CI, protoc must be installed (see .github/workflows/ci.yml).
    // Skip silently if the proto file is missing (e.g. partial checkouts).
    let proto = "../proto/parallax.proto";
    if std::path::Path::new(proto).exists() {
        tonic_build::configure()
            .build_server(false)
            .build_client(true)
            .compile_protos(&[proto], &["../proto"])
            .expect("Failed to compile protos — install protoc: https://grpc.io/docs/protoc-installation/");
    }

    tauri_build::build()
}
