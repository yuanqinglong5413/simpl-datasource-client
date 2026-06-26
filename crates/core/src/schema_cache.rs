use simpl_driver_trait::SchemaMeta;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 连接级 Schema 元数据缓存，供补全与 Schema 树复用。
pub struct SchemaCache {
    inner: Arc<RwLock<HashMap<Uuid, SchemaMeta>>>,
}

impl SchemaCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, connection_id: &Uuid) -> Option<SchemaMeta> {
        self.inner.read().await.get(connection_id).cloned()
    }

    pub async fn set(&self, connection_id: Uuid, meta: SchemaMeta) {
        self.inner.write().await.insert(connection_id, meta);
    }

    pub async fn invalidate(&self, connection_id: &Uuid) {
        self.inner.write().await.remove(connection_id);
    }
}

impl Default for SchemaCache {
    fn default() -> Self {
        Self::new()
    }
}
