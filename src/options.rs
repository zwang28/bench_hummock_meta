use clap::Parser;

#[derive(Parser, Debug)]
pub struct BenchmarkOptions {
    #[clap(long, default_value = "127.0.0.1:5690")]
    pub meta_address: String,
    #[clap(long, default_value = "127.0.0.1:5688")]
    pub host_address: String,
    #[clap(long, default_value = "100")]
    pub pin_version_interval_ms: u64,
    #[clap(long, default_value = "1000")]
    pub unpin_version_interval_ms: u64,
    #[clap(long, default_value = "1000")]
    pub heartbeat_interval_ms: u64,
    #[clap(long, default_value = "100")]
    pub checkpoint_interval_ms: u64,
}
