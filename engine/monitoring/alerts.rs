use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub acknowledged: bool,
}

pub struct AlertManager {
    alerts: Arc<Mutex<Vec<Alert>>>,
    next_alert_id: Arc<Mutex<u64>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            alerts: Arc::new(Mutex::new(Vec::new())),
            next_alert_id: Arc::new(Mutex::new(1)),
        }
    }

    pub async fn raise_alert(&self, severity: AlertSeverity, message: String) -> u64 {
        let mut next_id = self.next_alert_id.lock().await;
        let alert_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let alert = Alert {
            id: alert_id,
            severity: severity.clone(),
            message: message.clone(),
            timestamp: chrono::Utc::now(),
            acknowledged: false,
        };

        let mut alerts = self.alerts.lock().await;
        alerts.push(alert);

        tracing::warn!(
            "ALERT [{}]: {} - {}",
            match severity {
                AlertSeverity::Info => "INFO",
                AlertSeverity::Warning => "WARNING",
                AlertSeverity::Critical => "CRITICAL",
            },
            alert_id,
            message
        );

        alert_id
    }

    pub async fn acknowledge_alert(&self, alert_id: u64) -> anyhow::Result<()> {
        let mut alerts = self.alerts.lock().await;
        
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.acknowledged = true;
            Ok(())
        } else {
            anyhow::bail!("Alert not found: {}", alert_id)
        }
    }

    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.lock().await;
        alerts.iter()
            .filter(|a| !a.acknowledged)
            .cloned()
            .collect()
    }

    pub async fn get_critical_alerts(&self) -> Vec<Alert> {
        let alerts = self.alerts.lock().await;
        alerts.iter()
            .filter(|a| !a.acknowledged && a.severity == AlertSeverity::Critical)
            .cloned()
            .collect()
    }

    pub async fn clear_old_alerts(&self, hours: i64) {
        let mut alerts = self.alerts.lock().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours);
        
        alerts.retain(|a| a.timestamp > cutoff || !a.acknowledged);
    }
}
