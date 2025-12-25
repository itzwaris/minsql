use crate::ffi::storage::StorageEngine;
use crate::language::parser::Parser;
use crate::planner::logical::LogicalPlanner;
use crate::planner::physical::PhysicalPlanner;
use crate::execution::engine::ExecutionEngine;
use crate::replication::consensus::RaftNode;
use crate::telemetry::metrics::MetricsRegistry;
use crate::protocol::{handshake, Frame, MessageType};
use anyhow::Result;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

pub struct Server {
    port: u16,
    storage: Arc<StorageEngine>,
    raft_node: Arc<RaftNode>,
    metrics: Arc<MetricsRegistry>,
}

impl Server {
    pub fn new(
        port: u16,
        storage: Arc<StorageEngine>,
        raft_node: Arc<RaftNode>,
        metrics: Arc<MetricsRegistry>,
    ) -> Result<Self> {
        Ok(Self {
            port,
            storage,
            raft_node,
            metrics,
        })
    }

    pub async fn serve(self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("Server listening on {}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await?;
            tracing::info!("New connection from {}", peer_addr);

            let storage = self.storage.clone();
            let raft_node = self.raft_node.clone();
            let metrics = self.metrics.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, storage, raft_node, metrics).await {
                    tracing::error!("Connection error: {}", e);
                }
            });
        }
    }
}

async fn handle_connection(
    mut stream: TcpStream,
    storage: Arc<StorageEngine>,
    raft_node: Arc<RaftNode>,
    metrics: Arc<MetricsRegistry>,
) -> Result<()> {
    let _handshake_req = handshake::perform_handshake(&mut stream, raft_node.node_id()).await?;

    loop {
        let frame = Frame::read_from(&mut stream).await?;

        match frame.message_type {
            MessageType::Query => {
                let query_text = String::from_utf8(frame.payload)?;
                
                metrics.increment_queries();

                let response = match execute_query(&query_text, &storage).await {
                    Ok(result) => {
                        Frame::new(MessageType::QueryResponse, serde_json::to_vec(&result)?)
                    }
                    Err(e) => {
                        Frame::new(MessageType::Error, e.to_string().as_bytes().to_vec())
                    }
                };

                response.write_to(&mut stream).await?;
            }
            MessageType::Execute => {
                let statement = String::from_utf8(frame.payload)?;
                
                metrics.increment_executions();

                let response = match execute_statement(&statement, &storage, &raft_node).await {
                    Ok(()) => {
                        Frame::new(MessageType::ExecuteResponse, b"OK".to_vec())
                    }
                    Err(e) => {
                        Frame::new(MessageType::Error, e.to_string().as_bytes().to_vec())
                    }
                };

                response.write_to(&mut stream).await?;
            }
            _ => {
                tracing::warn!("Unexpected message type: {:?}", frame.message_type);
            }
        }
    }
}

async fn execute_query(query_text: &str, storage: &StorageEngine) -> Result<serde_json::Value> {
    let parser = Parser::new();
    let ast = parser.parse(query_text)?;

    let logical_planner = LogicalPlanner::new();
    let logical_plan = logical_planner.plan(&ast)?;

    let physical_planner = PhysicalPlanner::new(storage);
    let physical_plan = physical_planner.plan(&logical_plan)?;

    let mut execution_engine = ExecutionEngine::new(storage);
    let results = execution_engine.execute(physical_plan).await?;

    Ok(serde_json::json!({
        "rows": results,
    }))
}

async fn execute_statement(
    statement: &str,
    storage: &StorageEngine,
    raft_node: &RaftNode,
) -> Result<()> {
    let parser = Parser::new();
    let ast = parser.parse(statement)?;

    raft_node.propose_command(statement.as_bytes().to_vec()).await?;

    Ok(())
              }
