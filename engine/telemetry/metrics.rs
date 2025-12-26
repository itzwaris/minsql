use std::sync::atomic::{AtomicU64, Ordering};
use tokio::time::{Duration, interval};

pub struct MetricsRegistry {
    queries_executed: AtomicU64,
    statements_executed: AtomicU64,
    transactions_committed: AtomicU64,
    transactions_aborted: AtomicU64,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            queries_executed: AtomicU64::new(0),
            statements_executed: AtomicU64::new(0),
            transactions_committed: AtomicU64::new(0),
            transactions_aborted: AtomicU64::new(0),
        }
    }

    pub fn increment_queries(&self) {
        self.queries_executed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_executions(&self) {
        self.statements_executed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_commits(&self) {
        self.transactions_committed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_aborts(&self) {
        self.transactions_aborted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_queries(&self) -> u64 {
        self.queries_executed.load(Ordering::Relaxed)
    }

    pub fn get_executions(&self) -> u64 {
        self.statements_executed.load(Ordering::Relaxed)
    }

    pub async fn report_loop(&self) {
        let mut ticker = interval(Duration::from_secs(60));

        loop {
            ticker.tick().await;

            tracing::info!(
                "Metrics: queries={}, executions={}, commits={}, aborts={}",
                self.get_queries(),
                self.get_executions(),
                self.transactions_committed.load(Ordering::Relaxed),
                self.transactions_aborted.load(Ordering::Relaxed)
            );
        }
    }
}
