use crate::determinism::clock::LogicalTime;
use crate::transactions::manager::TransactionId;
use crate::transactions::snapshot::Snapshot;
use anyhow::Result;
use chrono::{DateTime, Utc};

pub struct TimeTravelManager;

impl TimeTravelManager {
    pub fn new() -> Self {
        Self
    }

    pub fn create_historical_snapshot(&self, timestamp: DateTime<Utc>) -> Result<Snapshot> {
        let logical_time = LogicalTime {
            logical: 0,
            physical: timestamp.timestamp_micros() as u64,
        };

        Ok(Snapshot::new(
            TransactionId(u64::MAX),
            logical_time,
            Vec::new(),
        ))
    }

    pub fn create_snapshot_at_logical_time(&self, logical_time: LogicalTime) -> Snapshot {
        Snapshot::new(
            TransactionId(u64::MAX),
            logical_time,
            Vec::new(),
        )
    }
}
