mod connection_manager;
mod driver_registry;
mod persistence;
mod query_engine;
mod query_history;
mod schema_cache;

pub use connection_manager::{
    ConnectionManager, ConnectionRecord, SaveConnectionRequest, TestConnectionRequest,
};
pub use driver_registry::ActiveDriver;
pub use persistence::PersistenceError;
pub use query_engine::QueryEngine;
pub use query_history::{QueryHistoryEntry, QueryHistoryStore};
pub use schema_cache::SchemaCache;

use std::path::PathBuf;

/// 应用数据目录默认值（可被环境变量 SIMPLSOURCE_DATA_DIR 覆盖）。
pub fn default_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("SIMPLSOURCE_DATA_DIR") {
        return PathBuf::from(dir);
    }
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".simplsource"))
        .join("SimplSource")
}
