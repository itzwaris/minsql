use crate::execution::tuple::Tuple;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

#[derive(Clone)]
struct CacheEntry {
    results: Vec<Tuple>,
    created_at: SystemTime,
    access_count: u64,
}

pub struct QueryCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    max_size: usize,
    ttl: Duration,
}

impl QueryCache {
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_size,
            ttl,
        }
    }

    pub async fn get(&self, query: &str) -> Option<Vec<Tuple>> {
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(query) {
            let age = SystemTime::now().duration_since(entry.created_at).ok()?;

            if age < self.ttl {
                entry.access_count += 1;
                return Some(entry.results.clone());
            } else {
                cache.remove(query);
            }
        }

        None
    }

    pub async fn put(&self, query: String, results: Vec<Tuple>) -> Result<()> {
        let mut cache = self.cache.write().await;

        if cache.len() >= self.max_size {
            self.evict_lru(&mut cache);
        }

        cache.insert(
            query,
            CacheEntry {
                results,
                created_at: SystemTime::now(),
                access_count: 0,
            },
        );

        Ok(())
    }

    pub async fn invalidate(&self, pattern: &str) -> Result<()> {
        let mut cache = self.cache.write().await;

        cache.retain(|key, _| !key.contains(pattern));

        Ok(())
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;

        let total_entries = cache.len();
        let total_accesses: u64 = cache.values().map(|e| e.access_count).sum();

        CacheStats {
            entries: total_entries,
            total_accesses,
            max_size: self.max_size,
        }
    }

    fn evict_lru(&self, cache: &mut HashMap<String, CacheEntry>) {
        if let Some(lru_key) = cache
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(k, _)| k.clone())
        {
            cache.remove(&lru_key);
        }
    }

    pub async fn cleanup_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(60));

        loop {
            interval.tick().await;

            let mut cache = self.cache.write().await;
            let now = SystemTime::now();

            cache.retain(|_, entry| {
                now.duration_since(entry.created_at)
                    .map(|age| age < self.ttl)
                    .unwrap_or(false)
            });
        }
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_accesses: u64,
    pub max_size: usize,
}
