fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().out_dir("./src/prost").compile(
        &[
            "proto/fake_compute_node.proto",
            "proto/fake_barrier_manager.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
