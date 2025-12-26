mod config;
mod lifecycle;
mod protocol;
mod language;
mod planner;
mod execution;
mod transactions;
mod determinism;
mod sharding;
mod replication;
mod udf;
mod ffi;
mod telemetry;
mod analytics;
mod security;
mod monitoring;
mod streams;
mod graphql;

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
