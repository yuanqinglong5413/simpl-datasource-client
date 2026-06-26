mod connect_resolve;
mod connection_manager;
mod dml_generator;
mod driver_registry;
mod export;
mod import;
mod persistence;
mod query_engine;
mod query_history;
mod schema_cache;
mod transaction_manager;

pub use connect_resolve::ResolvedConnection;
pub use connection_manager::{
    ConnectionManager, ConnectionRecord, SaveConnectionRequest, TestConnectionRequest,
};
pub use dml_generator::{CellChangeRequest, DmlPreview};
pub use driver_registry::ActiveDriver;
pub use export::{ExportFormat, ExportRequest, ExportResult};
pub use import::{ImportFileFormat, ImportMapping, ImportPreview, ImportRequest, ImportResult};
pub use persistence::PersistenceError;
pub use query_engine::QueryEngine;
pub use query_history::{QueryHistoryEntry, QueryHistoryStore};
pub use schema_cache::SchemaCache;
pub use transaction_manager::{TransactionState, TransactionStatus};

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
