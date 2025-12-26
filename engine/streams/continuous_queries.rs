use crate::execution::tuple::Tuple;
use crate::language::ast::Statement;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousQuery {
    pub id: String,
    pub name: String,
    pub query: Statement,
    pub source_table: String,
    pub window_type: WindowType,
    pub window_size: std::time::Duration,
    pub output_action: OutputAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowType {
    Tumbling,
    Sliding,
    Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputAction {
    InsertInto(String),
    Notify(String),
    Webhook(String),
}

pub struct ContinuousQueryEngine {
    queries: Arc<RwLock<HashMap<String, ContinuousQuery>>>,
    data_streams: Arc<RwLock<HashMap<String, mpsc::Sender<Tuple>>>>,
}

impl ContinuousQueryEngine {
    pub fn new() -> Self {
        Self {
            queries: Arc::new(RwLock::new(HashMap::new())),
            data_streams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register_continuous_query(&self, cq: ContinuousQuery) -> Result<()> {
        let query_id = cq.id.clone();
        let source_table = cq.source_table.clone();

        let (tx, mut rx) = mpsc::channel(1000);

        self.data_streams.write().await.insert(source_table.clone(), tx);

        let cq_clone = cq.clone();
        tokio::spawn(async move {
            let mut window_buffer = Vec::new();
            let mut window_start = std::time::Instant::now();

            while let Some(tuple) = rx.recv().await {
                window_buffer.push(tuple);

                let elapsed = window_start.elapsed();
                if elapsed >= cq_clone.window_size {
                    Self::process_window(&cq_clone, &window_buffer).await;
                    
                    match cq_clone.window_type {
                        WindowType::Tumbling => {
                            window_buffer.clear();
                            window_start = std::time::Instant::now();
                        }
                        WindowType::Sliding => {
                            let slide_amount = cq_clone.window_size / 2;
                            let cutoff = window_start + slide_amount;
                            window_start = std::time::Instant::now();
                        }
                        WindowType::Session => {
                            window_buffer.clear();
                            window_start = std::time::Instant::now();
                        }
                    }
                }
            }
        });

        self.queries.write().await.insert(query_id, cq);
        Ok(())
    }

    async fn process_window(cq: &ContinuousQuery, window: &[Tuple]) {
        tracing::info!(
            "Processing window for continuous query '{}': {} tuples",
            cq.name,
            window.len()
        );
    }

    pub async fn emit_to_stream(&self, table: &str, tuple: Tuple) -> Result<()> {
        let streams = self.data_streams.read().await;
        
        if let Some(tx) = streams.get(table) {
            tx.send(tuple).await.ok();
        }

        Ok(())
    }

    pub async fn remove_continuous_query(&self, query_id: &str) -> Result<()> {
        self.queries.write().await.remove(query_id);
        Ok(())
    }

    pub async fn list_continuous_queries(&self) -> Vec<ContinuousQuery> {
        self.queries.read().await.values().cloned().collect()
    }
  }
