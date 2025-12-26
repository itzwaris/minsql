use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub struct GraphQLSubscriptionManager {
    active_subscriptions: Arc<RwLock<HashMap<String, mpsc::Sender<Value>>>>,
}

impl GraphQLSubscriptionManager {
    pub fn new() -> Self {
        Self {
            active_subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn subscribe(&self, subscription_id: String) -> mpsc::Receiver<Value> {
        let (tx, rx) = mpsc::channel(100);

        self.active_subscriptions
            .write()
            .await
            .insert(subscription_id, tx);

        rx
    }

    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        self.active_subscriptions
            .write()
            .await
            .remove(subscription_id);
        Ok(())
    }

    pub async fn notify(&self, subscription_id: &str, data: Value) -> Result<()> {
        let subscriptions = self.active_subscriptions.read().await;

        if let Some(tx) = subscriptions.get(subscription_id) {
            tx.send(data).await.ok();
        }

        Ok(())
    }

    pub async fn broadcast(&self, data: Value) {
        let subscriptions = self.active_subscriptions.read().await;

        for (_, tx) in subscriptions.iter() {
            tx.send(data.clone()).await.ok();
        }
    }
}
