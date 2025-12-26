use crate::determinism::clock::LogicalTime;
use crate::transactions::snapshot::Snapshot;
use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub u64);

impl TransactionId {
    pub fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        TransactionId(COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionState {
    Active,
    Committed,
    Aborted,
}

pub struct Transaction {
    pub id: TransactionId,
    pub state: TransactionState,
    pub snapshot: Snapshot,
    pub logical_time: LogicalTime,
}

impl Transaction {
    pub fn new(snapshot: Snapshot, logical_time: LogicalTime) -> Self {
        Self {
            id: TransactionId::next(),
            state: TransactionState::Active,
            snapshot,
            logical_time,
        }
    }

    pub fn commit(&mut self) {
        self.state = TransactionState::Committed;
    }

    pub fn abort(&mut self) {
        self.state = TransactionState::Aborted;
    }
}

pub struct TransactionManager {
    active_transactions: Arc<DashMap<TransactionId, Transaction>>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            active_transactions: Arc::new(DashMap::new()),
        }
    }

    pub fn begin(&self, logical_time: LogicalTime) -> Result<TransactionId> {
        let active_xids: Vec<TransactionId> = self
            .active_transactions
            .iter()
            .map(|entry| *entry.key())
            .collect();

        let snapshot = Snapshot::new(TransactionId::next(), logical_time, active_xids);
        let transaction = Transaction::new(snapshot, logical_time);
        let xid = transaction.id;

        self.active_transactions.insert(xid, transaction);
        Ok(xid)
    }

    pub fn commit(&self, xid: TransactionId) -> Result<()> {
        if let Some(mut entry) = self.active_transactions.get_mut(&xid) {
            entry.commit();
        } else {
            anyhow::bail!("Transaction not found");
        }

        self.active_transactions.remove(&xid);
        Ok(())
    }

    pub fn abort(&self, xid: TransactionId) -> Result<()> {
        if let Some(mut entry) = self.active_transactions.get_mut(&xid) {
            entry.abort();
        } else {
            anyhow::bail!("Transaction not found");
        }

        self.active_transactions.remove(&xid);
        Ok(())
    }

    pub fn get_snapshot(&self, xid: TransactionId) -> Result<Snapshot> {
        self.active_transactions
            .get(&xid)
            .map(|entry| entry.snapshot.clone())
            .ok_or_else(|| anyhow::anyhow!("Transaction not found"))
    }
}
