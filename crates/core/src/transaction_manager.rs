use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 单连接未提交事务状态。
#[derive(Debug, Default)]
pub struct TransactionState {
    pub active: bool,
    pub pending_sql: Vec<String>,
}

/// 事务状态摘要（IPC 传输）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionStatus {
    pub active: bool,
    pub pending_count: usize,
}

/// 管理各连接的 Safe Mode 事务队列。
pub struct TransactionManager {
    sessions: HashMap<Uuid, TransactionState>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn status(&self, connection_id: &Uuid) -> TransactionStatus {
        let session = self.sessions.get(connection_id);
        TransactionStatus {
            active: session.map(|s| s.active).unwrap_or(false),
            pending_count: session.map(|s| s.pending_sql.len()).unwrap_or(0),
        }
    }

    pub fn begin_if_needed(&mut self, connection_id: Uuid) {
        self.sessions
            .entry(connection_id)
            .or_insert_with(|| TransactionState {
                active: true,
                pending_sql: vec![],
            });
    }

    pub fn queue_sql(&mut self, connection_id: Uuid, sql: String) {
        let session = self.sessions.entry(connection_id).or_default();
        session.active = true;
        session.pending_sql.push(sql);
    }

    pub fn pending_sql(&self, connection_id: &Uuid) -> Vec<String> {
        self.sessions
            .get(connection_id)
            .map(|s| s.pending_sql.clone())
            .unwrap_or_default()
    }

    pub fn clear(&mut self, connection_id: &Uuid) {
        self.sessions.remove(connection_id);
    }
}

impl Default for TransactionManager {
    fn default() -> Self {
        Self::new()
    }
}
