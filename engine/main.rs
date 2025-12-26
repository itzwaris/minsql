mod analytics;
mod config;
mod determinism;
mod execution;
mod ffi;
mod graphql;
mod language;
mod lifecycle;
mod monitoring;
mod planner;
mod protocol;
mod replication;
mod security;
mod sharding;
mod streams;
mod telemetry;
mod transactions;
mod udf;

use anyhow::Result;
use config::Config;
use lifecycle::Lifecycle;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let config = Config::from_args()?;

    tracing::info!("Starting minsql node {}", config.node_id);
    tracing::info!("Data directory: {}", config.data_dir);
    tracing::info!("Listening on port {}", config.port);
    tracing::info!("Streaming features enabled");
    tracing::info!("GraphQL API enabled");

    let lifecycle = Lifecycle::new(config).await?;
    lifecycle.run().await?;

    Ok(())
}
