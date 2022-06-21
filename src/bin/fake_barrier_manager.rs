use bench_hummock_meta::options::BenchmarkOptions;
use clap::Parser;
use rand::Rng;
use risingwave_hummock_sdk::HummockEpoch;
use risingwave_rpc_client::{HummockMetaClient, MetaClient};
use std::time::Duration;
use tokio::signal;
use tokio::task::JoinHandle;

use tracing::Level;

pub struct FakeBarrierManager {
    meta_client: MetaClient,
}

impl FakeBarrierManager {
    pub fn new(meta_client: MetaClient) -> Self {
        Self { meta_client }
    }

    pub async fn run(
        self,
        checkpoint_interval: Duration,
    ) -> (tokio::sync::oneshot::Sender<()>, JoinHandle<()>) {
        let (shutdown_send, mut shutdown_recv) = tokio::sync::oneshot::channel();
        let join_handle = tokio::spawn(async move {
            let mut epoch: HummockEpoch = 1;
            let mut checkpoint_ticker = tokio::time::interval(checkpoint_interval);
            checkpoint_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let sleep = rand::thread_rng().gen::<u64>() % 1000;
            tokio::time::sleep(Duration::from_millis(sleep)).await;
            loop {
                tokio::select! {
                    _ = checkpoint_ticker.tick() => {
                        // Commit with empty payload. Meta is hacked to fulfil it with correct SSTs.
                        if let Err(e) = self.meta_client.commit_epoch(epoch, vec![]).await {
                            tracing::warn!("{:#?}", e);
                            break;
                        }
                        epoch += 1;
                    },
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
    let checkpoint_interval_ms = options.checkpoint_interval_ms;

    let meta_client = MetaClient::new(meta_address)
        .await
        .expect("new meta client");
    let client = FakeBarrierManager::new(meta_client.clone());
    let (client_shutdown_send, client_join_handle) = client
        .run(Duration::from_millis(checkpoint_interval_ms))
        .await;

    tracing::info!("started..");

    match signal::ctrl_c().await {
        Ok(_) => {
            tracing::info!("shutting down..");
            client_shutdown_send.send(()).unwrap();
            client_join_handle.await.unwrap();
            tracing::info!("shutdown..");
        }
        Err(err) => {
            tracing::error!("{}", err);
        }
    }
}
