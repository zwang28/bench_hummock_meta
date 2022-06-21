use bench_hummock_meta::options::BenchmarkOptions;
use clap::Parser;
use itertools::Itertools;

use rand::Rng;
use risingwave_hummock_sdk::HummockVersionId;
use risingwave_pb::common::WorkerType;
use risingwave_rpc_client::{HummockMetaClient, MetaClient};
use std::collections::HashSet;

use std::time::Duration;
use tokio::signal;
use tokio::task::JoinHandle;

use tracing::Level;

pub struct FakeComputeNode {
    pinned_versions: HashSet<HummockVersionId>,
    meta_client: MetaClient,
}

impl FakeComputeNode {
    pub fn new(meta_client: MetaClient) -> Self {
        Self {
            pinned_versions: Default::default(),
            meta_client,
        }
    }

    pub async fn run(
        mut self,
        pin_interval: Duration,
        unpin_interval: Duration,
        checkpoint_interval: Duration,
    ) -> (tokio::sync::oneshot::Sender<()>, JoinHandle<()>) {
        let (shutdown_send, mut shutdown_recv) = tokio::sync::oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let mut pin_ticker = tokio::time::interval(pin_interval);
            pin_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut unpin_ticker = tokio::time::interval(unpin_interval);
            unpin_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut checkpoint_ticker = tokio::time::interval(checkpoint_interval);
            checkpoint_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let sleep = rand::thread_rng().gen::<u64>() % 1000;
            tokio::time::sleep(Duration::from_millis(sleep)).await;
            loop {
                tokio::select! {
                    _ = pin_ticker.tick() => {
                        match self.meta_client.pin_version(HummockVersionId::MAX).await {
                            Ok(resp) => {self.pinned_versions.insert(resp.id);},
                            Err(e) => {
                                tracing::warn!("{:#?}", e);
                                break;
                            }
                        }
                    },
                    _ = unpin_ticker.tick() => {
                        if let Err(e) = self.meta_client.unpin_version(&self.pinned_versions.drain().collect_vec()).await {
                            tracing::warn!("{:#?}", e);
                            break;
                        }
                    },
                    _ = checkpoint_ticker.tick() => {
                        if let Err(e) = self.meta_client.get_new_table_id().await {
                            tracing::warn!("{:#?}", e);
                            break;
                        }
                    }
                    _ = &mut shutdown_recv => {
                        break;
                    }
                }
            }
            tracing::info!("client shutting down");
        });
        (shutdown_send, join_handle)
    }
}

#[tokio::main]
async fn main() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let options: BenchmarkOptions = BenchmarkOptions::parse();

    let meta_address = &["http://", &options.meta_address].concat();
    let host_address = options.host_address;
    let heartbeat_interval_ms = options.heartbeat_interval_ms;
    let pin_version_interval_ms = options.pin_version_interval_ms;
    let unpin_version_interval_ms = options.unpin_version_interval_ms;
    let checkpoint_interval_ms = options.checkpoint_interval_ms;

    let mut meta_client = MetaClient::new(meta_address)
        .await
        .expect("new meta client");
    meta_client
        .register(&host_address.parse().unwrap(), WorkerType::ComputeNode)
        .await
        .expect("register meta client");
    let (heartbeat_join_handle, heartbeat_shutdown_send) = MetaClient::start_heartbeat_loop(
        meta_client.clone(),
        Duration::from_millis(heartbeat_interval_ms),
    );

    let client = FakeComputeNode::new(meta_client.clone());
    let (client_shutdown_send, client_join_handle) = client
        .run(
            Duration::from_millis(pin_version_interval_ms),
            Duration::from_millis(unpin_version_interval_ms),
            Duration::from_millis(checkpoint_interval_ms),
        )
        .await;

    tracing::info!("started..");

    match signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("shutting down..");
            client_shutdown_send.send(()).unwrap();
            heartbeat_shutdown_send.send(()).unwrap();
            client_join_handle.await.unwrap();
            heartbeat_join_handle.await.unwrap();
            tracing::info!("shutdown..");
        }
        Err(err) => {
            tracing::error!("{}", err);
        }
    }
}
