fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use bundled protoc if system protoc is not available
    std::env::set_var("PROTOC", protobuf_src::protoc());

    tonic_build::compile_protos("../../proto/auth.proto")?;
    Ok(())
}
