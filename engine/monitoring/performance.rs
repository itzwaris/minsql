use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformance {
    pub query: String,
    pub execution_time: Duration,
    pub rows_returned: usize,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub avg_query_time: Duration,
    pub p50_query_time: Duration,
    pub p95_query_time: Duration,
    pub p99_query_time: Duration,
    pub slowest_queries: Vec<QueryPerformance>,
    pub total_queries: usize,
}

pub struct PerformanceMonitor {
    recent_queries: Arc<Mutex<VecDeque<QueryPerformance>>>,
    max_history: usize,
}

impl PerformanceMonitor {
    pub fn new(max_history: usize) -> Self {
        Self {
            recent_queries: Arc::new(Mutex::new(VecDeque::with_capacity(max_history))),
            max_history,
        }
    }

    pub async fn record_query(
        &self,
        query: String,
        execution_time: Duration,
        rows_returned: usize,
    ) {
        let mut queries = self.recent_queries.lock().await;

        let perf = QueryPerformance {
            query,
            execution_time,
            rows_returned,
            timestamp: chrono::Utc::now(),
        };

        if queries.len() >= self.max_history {
            queries.pop_front();
        }

        queries.push_back(perf);
    }

    pub async fn get_stats(&self) -> PerformanceStats {
        let queries = self.recent_queries.lock().await;

        if queries.is_empty() {
            return PerformanceStats {
                avg_query_time: Duration::from_secs(0),
                p50_query_time: Duration::from_secs(0),
                p95_query_time: Duration::from_secs(0),
                p99_query_time: Duration::from_secs(0),
                slowest_queries: Vec::new(),
                total_queries: 0,
            };
        }

        let mut times: Vec<Duration> = queries.iter().map(|q| q.execution_time).collect();
        times.sort();

        let total: Duration = times.iter().sum();
        let avg = total / times.len() as u32;

        let p50_idx = times.len() / 2;
        let p95_idx = (times.len() as f64 * 0.95) as usize;
        let p99_idx = (times.len() as f64 * 0.99) as usize;

        let p50 = times
            .get(p50_idx)
            .copied()
            .unwrap_or(Duration::from_secs(0));
        let p95 = times
            .get(p95_idx)
            .copied()
            .unwrap_or(Duration::from_secs(0));
        let p99 = times
            .get(p99_idx)
            .copied()
            .unwrap_or(Duration::from_secs(0));

        let mut sorted_queries: Vec<QueryPerformance> = queries.iter().cloned().collect();
        sorted_queries.sort_by(|a, b| b.execution_time.cmp(&a.execution_time));
        let slowest = sorted_queries.into_iter().take(10).collect();

        PerformanceStats {
            avg_query_time: avg,
            p50_query_time: p50,
            p95_query_time: p95,
            p99_query_time: p99,
            slowest_queries: slowest,
            total_queries: queries.len(),
        }
    }

    pub async fn get_slow_queries(&self, threshold: Duration) -> Vec<QueryPerformance> {
        let queries = self.recent_queries.lock().await;

        queries
            .iter()
            .filter(|q| q.execution_time > threshold)
            .cloned()
            .collect()
    }

    pub async fn clear_history(&self) {
        let mut queries = self.recent_queries.lock().await;
        queries.clear();
    }
}
