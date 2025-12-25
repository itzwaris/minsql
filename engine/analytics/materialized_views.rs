use crate::execution::tuple::Tuple;
use crate::language::ast::Statement;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MaterializedView {
    pub name: String,
    pub query: Statement,
    pub data: Vec<Tuple>,
    pub last_refresh: std::time::SystemTime,
}

pub struct MaterializedViewManager {
    views: Arc<RwLock<HashMap<String, MaterializedView>>>,
}

impl MaterializedViewManager {
    pub fn new() -> Self {
        Self {
            views: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_view(&self, name: String, query: Statement) -> Result<()> {
        let view = MaterializedView {
            name: name.clone(),
            query,
            data: Vec::new(),
            last_refresh: std::time::SystemTime::now(),
        };

        let mut views = self.views.write().await;
        views.insert(name, view);
        Ok(())
    }

    pub async fn refresh_view(&self, name: &str) -> Result<()> {
        let mut views = self.views.write().await;
        
        if let Some(view) = views.get_mut(name) {
            view.last_refresh = std::time::SystemTime::now();
            Ok(())
        } else {
            anyhow::bail!("Materialized view not found: {}", name)
        }
    }

    pub async fn query_view(&self, name: &str) -> Result<Vec<Tuple>> {
        let views = self.views.read().await;
        
        if let Some(view) = views.get(name) {
            Ok(view.data.clone())
        } else {
            anyhow::bail!("Materialized view not found: {}", name)
        }
    }

    pub async fn drop_view(&self, name: &str) -> Result<()> {
        let mut views = self.views.write().await;
        
        if views.remove(name).is_some() {
            Ok(())
        } else {
            anyhow::bail!("Materialized view not found: {}", name)
        }
    }

    pub async fn list_views(&self) -> Vec<String> {
        let views = self.views.read().await;
        views.keys().cloned().collect()
    }

    pub async fn auto_refresh_loop(self: Arc<Self>) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));

        loop {
            interval.tick().await;

            let view_names = self.list_views().await;
            
            for name in view_names {
                if let Err(e) = self.refresh_view(&name).await {
                    tracing::error!("Failed to refresh view {}: {}", name, e);
                }
            }
        }
    }
}
