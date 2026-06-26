use serde::{Deserialize, Serialize};
use simpl_driver_trait::ConnectionConfig;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

const CONNECTIONS_FILE: &str = "connections.json";

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("read failed: {0}")]
    Read(String),
    #[error("write failed: {0}")]
    Write(String),
    #[error("connection not found")]
    NotFound,
}

/// 持久化连接列表（不含密码）。
pub struct ConnectionPersistence {
    data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConnectionStore {
    connections: Vec<ConnectionConfig>,
}

impl ConnectionPersistence {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        let _ = fs::create_dir_all(&data_dir);
        Self { data_dir }
    }

    fn file_path(&self) -> PathBuf {
        self.data_dir.join(CONNECTIONS_FILE)
    }

    pub fn load_all(&self) -> Result<Vec<ConnectionConfig>, PersistenceError> {
        let path = self.file_path();
        if !path.exists() {
            return Ok(vec![]);
        }
        let raw = fs::read_to_string(&path).map_err(|e| PersistenceError::Read(e.to_string()))?;
        let store: ConnectionStore =
            serde_json::from_str(&raw).map_err(|e| PersistenceError::Read(e.to_string()))?;
        Ok(store.connections)
    }

    pub fn save_all(&self, connections: &[ConnectionConfig]) -> Result<(), PersistenceError> {
        let store = ConnectionStore {
            connections: connections.to_vec(),
        };
        let json =
            serde_json::to_string_pretty(&store).map_err(|e| PersistenceError::Write(e.to_string()))?;
        fs::write(self.file_path(), json).map_err(|e| PersistenceError::Write(e.to_string()))
    }

    pub fn get(&self, id: &Uuid) -> Result<ConnectionConfig, PersistenceError> {
        self.load_all()?
            .into_iter()
            .find(|c| &c.id == id)
            .ok_or(PersistenceError::NotFound)
    }
}
