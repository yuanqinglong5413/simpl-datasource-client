use simpl_datasource_core::{
    ConnectionManager, QueryEngine, QueryHistoryStore, SchemaCache,
};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 全局应用状态，由 Tauri 管理。
pub struct AppState {
    pub connections: Arc<ConnectionManager>,
    pub schema_cache: Arc<SchemaCache>,
    pub query_engine: Arc<QueryEngine>,
    pub history: Arc<QueryHistoryStore>,
    /// 异步命令串行化锁，避免并发写连接文件。
    pub io_lock: Mutex<()>,
}

impl AppState {
    pub fn new(data_dir: std::path::PathBuf) -> Self {
        let connections = Arc::new(ConnectionManager::new(&data_dir));
        let schema_cache = Arc::new(SchemaCache::new());
        let query_engine = Arc::new(QueryEngine::new(
            connections.clone(),
            schema_cache.clone(),
        ));
        let history = Arc::new(QueryHistoryStore::new(&data_dir));
        Self {
            connections,
            schema_cache,
            query_engine,
            history,
            io_lock: Mutex::new(()),
        }
    }
}
