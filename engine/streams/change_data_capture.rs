use crate::execution::tuple::Tuple;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    pub change_id: u64,
    pub change_type: ChangeType,
    pub table: String,
    pub before: Option<Tuple>,
    pub after: Option<Tuple>,
    pub timestamp: DateTime<Utc>,
    pub transaction_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDCSubscription {
    pub id: String,
    pub tables: Vec<String>,
    pub operations: Vec<ChangeType>,
    pub filter: Option<String>,
}

pub struct ChangeDataCapture {
    subscribers: Arc<RwLock<HashMap<String, mpsc::Sender<ChangeEvent>>>>,
    subscriptions: Arc<RwLock<HashMap<String, CDCSubscription>>>,
    next_change_id: Arc<RwLock<u64>>,
}

impl ChangeDataCapture {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            next_change_id: Arc::new(RwLock::new(1)),
        }
    }

    pub async fn subscribe(&self, subscription: CDCSubscription) -> Result<mpsc::Receiver<ChangeEvent>> {
        let (tx, rx) = mpsc::channel(1000);

        self.subscribers.write().await.insert(subscription.id.clone(), tx);
        self.subscriptions.write().await.insert(subscription.id.clone(), subscription);

        Ok(rx)
    }

    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        self.subscribers.write().await.remove(subscription_id);
        self.subscriptions.write().await.remove(subscription_id);
        Ok(())
    }

    pub async fn emit_change(
        &self,
        change_type: ChangeType,
        table: String,
        before: Option<Tuple>,
        after: Option<Tuple>,
        transaction_id: u64,
    ) -> Result<()> {
        let mut next_id = self.next_change_id.write().await;
        let change_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let event = ChangeEvent {
            change_id,
            change_type: change_type.clone(),
            table: table.clone(),
            before,
            after,
            timestamp: Utc::now(),
            transaction_id,
        };

        let subscriptions = self.subscriptions.read().await;
        let subscribers = self.subscribers.read().await;

        for (sub_id, subscription) in subscriptions.iter() {
            if !subscription.tables.contains(&table) {
                continue;
            }

            if !Self::matches_operation(&subscription.operations, &change_type) {
                continue;
            }

            if let Some(tx) = subscribers.get(sub_id) {
                tx.send(event.clone()).await.ok();
            }
        }

        Ok(())
    }

    fn matches_operation(operations: &[ChangeType], change_type: &ChangeType) -> bool {
        if operations.is_empty() {
            return true;
        }

        operations.iter().any(|op| std::mem::discriminant(op) == std::mem::discriminant(change_type))
    }

    pub async fn get_change_log(
        &self,
        table: Option<String>,
        since: Option<DateTime<Utc>>,
        limit: usize,
    ) -> Vec<ChangeEvent> {
        Vec::new()
    }

    pub async fn export_changes(
        &self,
        format: &str,
        table: Option<String>,
    ) -> Result<String> {
        match format {
            "json" => Ok(serde_json::to_string_pretty(&Vec::<ChangeEvent>::new())?),
            "csv" => Ok("change_id,change_type,table,timestamp\n".to_string()),
            _ => anyhow::bail!("Unsupported format: {}", format),
        }
    }

    pub async fn list_subscriptions(&self) -> Vec<CDCSubscription> {
        self.subscriptions.read().await.values().cloned().collect()
    }
          }
