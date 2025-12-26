use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub channel: String,
    pub payload: Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub headers: HashMap<String, String>,
}

pub struct PubSubBroker {
    channels: Arc<RwLock<HashMap<String, Vec<mpsc::Sender<Message>>>>>,
    message_history: Arc<RwLock<Vec<Message>>>,
    max_history: usize,
}

impl PubSubBroker {
    pub fn new(max_history: usize) -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    pub async fn subscribe(&self, channel: &str) -> mpsc::Receiver<Message> {
        let (tx, rx) = mpsc::channel(1000);

        let mut channels = self.channels.write().await;
        channels
            .entry(channel.to_string())
            .or_insert_with(Vec::new)
            .push(tx);

        rx
    }

    pub async fn publish(&self, channel: &str, payload: Value) -> Result<()> {
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            channel: channel.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
            headers: HashMap::new(),
        };

        self.store_message(message.clone()).await;

        let channels = self.channels.read().await;

        if let Some(subscribers) = channels.get(channel) {
            for subscriber in subscribers {
                let _ = subscriber.send(message.clone()).await;
            }
        }

        Ok(())
    }

    pub async fn publish_with_headers(
        &self,
        channel: &str,
        payload: Value,
        headers: HashMap<String, String>,
    ) -> Result<()> {
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            channel: channel.to_string(),
            payload,
            timestamp: chrono::Utc::now(),
            headers,
        };

        self.store_message(message.clone()).await;

        let channels = self.channels.read().await;

        if let Some(subscribers) = channels.get(channel) {
            for subscriber in subscribers {
                let _ = subscriber.send(message.clone()).await;
            }
        }

        Ok(())
    }

    async fn store_message(&self, message: Message) {
        let mut history = self.message_history.write().await;

        if history.len() >= self.max_history {
            history.remove(0);
        }

        history.push(message);
    }

    pub async fn get_message_history(
        &self,
        channel: Option<String>,
        limit: usize,
    ) -> Vec<Message> {
        let history = self.message_history.read().await;

        history
            .iter()
            .filter(|m| channel.as_ref().is_none_or(|c| &m.channel == c))
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn list_channels(&self) -> Vec<String> {
        self.channels.read().await.keys().cloned().collect()
    }

    pub async fn subscriber_count(&self, channel: &str) -> usize {
        self.channels
            .read()
            .await
            .get(channel)
            .map(|subs| subs.len())
            .unwrap_or(0)
    }

    pub async fn cleanup_dead_subscribers(&self) {
        let mut channels = self.channels.write().await;

        for (_, subscribers) in channels.iter_mut() {
            subscribers.retain(|sub| !sub.is_closed());
        }
    }
}
