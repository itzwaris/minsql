use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub event_id: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub event_type: String,
    pub event_data: Value,
    pub timestamp: DateTime<Utc>,
    pub version: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregate {
    pub id: String,
    pub aggregate_type: String,
    pub version: u64,
    pub state: Value,
}

pub struct EventStore {
    events: Arc<RwLock<Vec<Event>>>,
    aggregates: Arc<RwLock<HashMap<String, Aggregate>>>,
    snapshots: Arc<RwLock<HashMap<String, (u64, Value)>>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
            aggregates: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn append_event(&self, event: Event) -> Result<()> {
        let mut aggregates = self.aggregates.write().await;

        let aggregate = aggregates
            .entry(event.aggregate_id.clone())
            .or_insert_with(|| Aggregate {
                id: event.aggregate_id.clone(),
                aggregate_type: event.aggregate_type.clone(),
                version: 0,
                state: Value::Null,
            });

        if event.version != aggregate.version + 1 {
            anyhow::bail!(
                "Version mismatch: expected {}, got {}",
                aggregate.version + 1,
                event.version
            );
        }

        aggregate.version = event.version;

        let mut events = self.events.write().await;
        events.push(event);

        Ok(())
    }

    pub async fn get_events(&self, aggregate_id: &str, from_version: Option<u64>) -> Vec<Event> {
        let events = self.events.read().await;

        events
            .iter()
            .filter(|e| {
                e.aggregate_id == aggregate_id && from_version.is_none_or(|v| e.version >= v)
            })
            .cloned()
            .collect()
    }

    pub async fn get_aggregate_state(&self, aggregate_id: &str) -> Option<Aggregate> {
        self.aggregates.read().await.get(aggregate_id).cloned()
    }

    pub async fn create_snapshot(
        &self,
        aggregate_id: String,
        version: u64,
        state: Value,
    ) -> Result<()> {
        self.snapshots
            .write()
            .await
            .insert(aggregate_id, (version, state));
        Ok(())
    }

    pub async fn get_snapshot(&self, aggregate_id: &str) -> Option<(u64, Value)> {
        self.snapshots.read().await.get(aggregate_id).cloned()
    }

    pub async fn rebuild_aggregate(&self, aggregate_id: &str) -> Result<Value> {
        if let Some((snapshot_version, snapshot_state)) = self.get_snapshot(aggregate_id).await {
            let events = self
                .get_events(aggregate_id, Some(snapshot_version + 1))
                .await;

            let mut state = snapshot_state;
            for event in events {
                state = self.apply_event(state, &event);
            }

            Ok(state)
        } else {
            let events = self.get_events(aggregate_id, None).await;

            let mut state = Value::Null;
            for event in events {
                state = self.apply_event(state, &event);
            }

            Ok(state)
        }
    }

    fn apply_event(&self, state: Value, _event: &Event) -> Value {
        state
    }

    pub async fn get_event_stream(
        &self,
        aggregate_type: Option<String>,
        from_timestamp: Option<DateTime<Utc>>,
    ) -> Vec<Event> {
        let events = self.events.read().await;

        events
            .iter()
            .filter(|e| {
                aggregate_type
                    .as_ref()
                    .is_none_or(|t| &e.aggregate_type == t)
                    && from_timestamp.is_none_or(|ts| e.timestamp >= ts)
            })
            .cloned()
            .collect()
    }

    pub async fn purge_old_events(&self, before: DateTime<Utc>) -> Result<usize> {
        let mut events = self.events.write().await;
        let original_len = events.len();

        events.retain(|e| e.timestamp >= before);

        Ok(original_len - events.len())
    }
}
