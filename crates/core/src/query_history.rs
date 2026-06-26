use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const HISTORY_FILE: &str = "query_history.json";
const MAX_ENTRIES: usize = 500;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryHistoryEntry {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub connection_name: String,
    pub sql: String,
    pub executed_at: DateTime<Utc>,
    pub duration_ms: u64,
    pub row_count: u64,
    pub favorite: bool,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct HistoryStore {
    entries: Vec<QueryHistoryEntry>,
}

/// 本地查询历史存储。
pub struct QueryHistoryStore {
    path: PathBuf,
}

impl QueryHistoryStore {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let path = data_dir.as_ref().join(HISTORY_FILE);
        Self { path }
    }

    fn load(&self) -> HistoryStore {
        if !self.path.exists() {
            return HistoryStore::default();
        }
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    fn save(&self, store: &HistoryStore) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
        fs::write(&self.path, json).map_err(|e| e.to_string())
    }

    pub fn append(&self, entry: QueryHistoryEntry) -> Result<(), String> {
        let mut store = self.load();
        store.entries.insert(0, entry);
        if store.entries.len() > MAX_ENTRIES {
            store.entries.truncate(MAX_ENTRIES);
        }
        self.save(&store)
    }

    pub fn list(&self, connection_id: Option<Uuid>, limit: usize) -> Vec<QueryHistoryEntry> {
        let store = self.load();
        store
            .entries
            .into_iter()
            .filter(|e| connection_id.map(|id| e.connection_id == id).unwrap_or(true))
            .take(limit)
            .collect()
    }

    pub fn toggle_favorite(&self, id: &Uuid) -> Result<bool, String> {
        let mut store = self.load();
        let entry = store
            .entries
            .iter_mut()
            .find(|e| &e.id == id)
            .ok_or_else(|| "history entry not found".to_string())?;
        entry.favorite = !entry.favorite;
        let fav = entry.favorite;
        self.save(&store)?;
        Ok(fav)
    }
}
