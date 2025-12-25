use crate::config::Config;
use crate::ffi::storage::StorageEngine;
use crate::protocol::server::Server;
use crate::replication::consensus::RaftNode;
use crate::telemetry::metrics::MetricsRegistry;
use anyhow::Result;
use std::sync::Arc;
use tokio::signal;

pub struct Lifecycle {
    config: Config,
    storage: Arc<StorageEngine>,
    server: Server,
    raft_node: Arc<RaftNode>,
    metrics: Arc<MetricsRegistry>,
}

impl Lifecycle {
    pub async fn new(config: Config) -> Result<Self> {
        let storage = Arc::new(StorageEngine::new(&config.data_dir)?);
        
        storage.recover()?;

        let metrics = Arc::new(MetricsRegistry::new());

        let raft_node = Arc::new(RaftNode::new(
            config.node_id,
            config.peers.clone(),
            storage.clone(),
        )?);

        let server = Server::new(
            config.port,
            storage.clone(),
            raft_node.clone(),
            metrics.clone(),
        )?;

        Ok(Self {
            config,
            storage,
            server,
            raft_node,
            metrics,
        })
    }

    pub async fn run(self) -> Result<()> {
        tracing::info!("minsql node {} starting", self.config.node_id);

        let raft_handle = {
            let raft_node = self.raft_node.clone();
            tokio::spawn(async move {
                raft_node.run().await
            })
        };

        let server_handle = {
            let server = self.server;
            tokio::spawn(async move {
                server.serve().await
            })
        };

        let metrics_handle = {
            let metrics = self.metrics.clone();
            tokio::spawn(async move {
                metrics.report_loop().await
            })
        };

        tokio::select! {
            result = raft_handle => {
                tracing::error!("Raft node exited: {:?}", result);
            }
            result = server_handle => {
                tracing::error!("Server exited: {:?}", result);
            }
            result = metrics_handle => {
                tracing::error!("Metrics exited: {:?}", result);
            }
            _ = signal::ctrl_c() => {
                tracing::info!("Received shutdown signal");
            }
        }

        self.shutdown().await?;
        Ok(())
    }

    async fn shutdown(self) -> Result<()> {
        tracing::info!("Shutting down node {}", self.config.node_id);

        self.storage.checkpoint()?;
        self.storage.shutdown();

        tracing::info!("Shutdown complete");
        Ok(())
    }
}
