
fn main() -> Result<(), Box<dyn std::error::Error>> {
  tonic_build::compile_protos("src/proto/centre.proto")?;
  tonic_build::compile_protos("src/proto/platform.proto")?;
  Ok(())
}