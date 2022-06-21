use std::time::Duration;
use risingwave_common::util::addr::HostAddr;
use risingwave_pb::common::WorkerType;
use risingwave_rpc_client::MetaClient;
use tokio::signal;
use tracing::instrument::WithSubscriber;
use tracing::Level;

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // TODO: configurable
    let meta_address = "http://127.0.0.1:5690";
    let host_address: HostAddr = "127.0.0.1:123".parse().unwrap();
    let heartbeat_interval_ms = 1000;

    tracing::info!("starting..");
    let mut meta_client = MetaClient::new(meta_address).await.expect("new meta client");
    tracing::info!("registering..");
    meta_client.register(&host_address, WorkerType::ComputeNode).await.expect("register meta client");
    let (join_handle, shutdown_sender) = MetaClient::start_heartbeat_loop(
        meta_client.clone(),
        Duration::from_millis(heartbeat_interval_ms as u64),
    );
    tracing::info!("started..");

    match signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("shutting down..");
            shutdown_sender.send(()).unwrap();
            join_handle.await.unwrap();
            tracing::info!("shutdown..");
        }
        Err(err) => {
            tracing::error!("{}", err);
        }
    }
}
