use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    QueryExecution,
    DataModification,
    SchemaChange,
    Authentication,
    Authorization,
    ConfigurationChange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: u64,
    pub event_type: AuditEventType,
    pub timestamp: DateTime<Utc>,
    pub user: String,
    pub query: Option<String>,
    pub table: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub ip_address: Option<String>,
}

pub struct AuditLogger {
    events: Arc<Mutex<Vec<AuditEvent>>>,
    next_event_id: Arc<Mutex<u64>>,
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
            next_event_id: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn log_event(&self, mut event: AuditEvent) -> Result<()> {
        let mut next_id = self.next_event_id.lock().await;
        event.event_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let mut events = self.events.lock().await;
        events.push(event.clone());

        tracing::info!(
            "AUDIT: event_id={}, type={:?}, user={}, success={}",
            event.event_id,
            event.event_type,
            event.user,
            event.success
        );

        Ok(())
    }

    pub async fn log_query(
        &self,
        user: String,
        query: String,
        success: bool,
        error: Option<String>,
    ) -> Result<()> {
        let event = AuditEvent {
            event_id: 0,
            event_type: AuditEventType::QueryExecution,
            timestamp: Utc::now(),
            user,
            query: Some(query),
            table: None,
            success,
            error_message: error,
            ip_address: None,
        };

        self.log_event(event).await
    }

    pub async fn log_authentication(
        &self,
        user: String,
        success: bool,
        ip_address: Option<String>,
    ) -> Result<()> {
        let event = AuditEvent {
            event_id: 0,
            event_type: AuditEventType::Authentication,
            timestamp: Utc::now(),
            user,
            query: None,
            table: None,
            success,
            error_message: None,
            ip_address,
        };

        self.log_event(event).await
    }

    pub async fn log_schema_change(
        &self,
        user: String,
        query: String,
        table: String,
    ) -> Result<()> {
        let event = AuditEvent {
            event_id: 0,
            event_type: AuditEventType::SchemaChange,
            timestamp: Utc::now(),
            user,
            query: Some(query),
            table: Some(table),
            success: true,
            error_message: None,
            ip_address: None,
        };

        self.log_event(event).await
    }

    pub async fn query_logs(
        &self,
        user: Option<String>,
        event_type: Option<AuditEventType>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
    ) -> Vec<AuditEvent> {
        let events = self.events.lock().await;

        events
            .iter()
            .filter(|e| {
                if let Some(ref u) = user {
                    if e.user != *u {
                        return false;
                    }
                }

                if let Some(ref et) = event_type {
                    if std::mem::discriminant(&e.event_type) != std::mem::discriminant(et) {
                        return false;
                    }
                }

                if let Some(start) = start_time {
                    if e.timestamp < start {
                        return false;
                    }
                }

                if let Some(end) = end_time {
                    if e.timestamp > end {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    pub async fn export_logs(&self, format: &str) -> Result<String> {
        let events = self.events.lock().await;

        match format {
            "json" => Ok(serde_json::to_string_pretty(&*events)?),
            "csv" => {
                let mut csv = String::from("event_id,event_type,timestamp,user,success\n");
                for event in events.iter() {
                    csv.push_str(&format!(
                        "{},{:?},{},{},{}\n",
                        event.event_id,
                        event.event_type,
                        event.timestamp,
                        event.user,
                        event.success
                    ));
                }
                Ok(csv)
            }
            _ => anyhow::bail!("Unsupported format: {}", format),
        }
    }
}
