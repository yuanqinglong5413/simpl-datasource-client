use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 支持的数据库种类（MVP 三库）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseKind {
    Postgres,
    Mysql,
    Sqlite,
}

impl DatabaseKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Postgres => "PostgreSQL",
            Self::Mysql => "MySQL",
            Self::Sqlite => "SQLite",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            Self::Postgres => 5432,
            Self::Mysql => 3306,
            Self::Sqlite => 0,
        }
    }
}

/// SQL 方言标识，用于补全与 DDL 生成。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dialect {
    Postgres,
    Mysql,
    Sqlite,
}

impl From<DatabaseKind> for Dialect {
    fn from(kind: DatabaseKind) -> Self {
        match kind {
            DatabaseKind::Postgres => Self::Postgres,
            DatabaseKind::Mysql => Self::Mysql,
            DatabaseKind::Sqlite => Self::Sqlite,
        }
    }
}

/// SSL 连接模式。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SslMode {
    #[default]
    Prefer,
    Require,
    Disable,
    VerifyCa,
}

/// SSH 隧道配置（Phase 1 基础字段，U8 实现隧道逻辑）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub username: String,
    /// 密钥路径；密码通过 CredentialStore 单独保存。
    pub private_key_path: Option<String>,
}

impl Default for SshConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: String::new(),
            port: 22,
            username: String::new(),
            private_key_path: None,
        }
    }
}

/// 连接配置（密码/SSH 密码不持久化在此结构，由 credential 模块管理）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: Uuid,
    pub name: String,
    pub kind: DatabaseKind,
    pub host: String,
    pub port: u16,
    /// SQLite 文件路径；关系型库可为空。
    pub database: String,
    pub username: String,
    pub ssl_mode: SslMode,
    pub ssh: SshConfig,
    pub read_only: bool,
    pub environment: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConnectionConfig {
    pub fn new(name: impl Into<String>, kind: DatabaseKind) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            kind,
            host: "127.0.0.1".to_string(),
            port: kind.default_port(),
            database: String::new(),
            username: String::new(),
            ssl_mode: SslMode::default(),
            ssh: SshConfig::default(),
            read_only: false,
            environment: None,
            color: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// 连接测试/执行时携带的密钥材料（仅内存，不序列化落盘）。
#[derive(Debug, Clone, Default)]
pub struct ConnectionSecrets {
    pub password: Option<String>,
    pub ssh_password: Option<String>,
}

/// Schema 元数据根节点。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchemaMeta {
    pub databases: Vec<DatabaseMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseMeta {
    pub name: String,
    pub schemas: Vec<SchemaNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaNode {
    pub name: String,
    pub tables: Vec<TableMeta>,
    pub views: Vec<TableMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMeta {
    pub schema: String,
    pub name: String,
    pub columns: Vec<ColumnMeta>,
    pub primary_key: Vec<String>,
    pub row_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

/// 单元格值，跨 IPC 传输。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CellValue {
    Null,
    Bool(bool),
    Int64(i64),
    Float64(f64),
    String(String),
    Bytes(String),
    Json(serde_json::Value),
    DateTime(String),
}

/// 分页请求。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePageRequest {
    pub schema: Option<String>,
    pub table: String,
    pub offset: u64,
    pub limit: u64,
    pub order_by: Option<String>,
    pub order_desc: bool,
    pub filter: Option<String>,
}

/// 分页结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowPage {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<CellValue>>,
    pub total_rows: Option<i64>,
    pub offset: u64,
    pub limit: u64,
}

/// 查询执行结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub columns: Vec<ColumnMeta>,
    pub rows: Vec<Vec<CellValue>>,
    pub rows_affected: u64,
    pub execution_time_ms: u64,
    pub message: Option<String>,
}

/// EXPLAIN 计划行。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRow {
    pub fields: Vec<(String, CellValue)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlan {
    pub rows: Vec<ExplainRow>,
    pub raw: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_config_roundtrip_json() {
        let cfg = ConnectionConfig::new("本地 PG", DatabaseKind::Postgres);
        let json = serde_json::to_string(&cfg).expect("serialize");
        let back: ConnectionConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.name, "本地 PG");
        assert_eq!(back.kind, DatabaseKind::Postgres);
    }
}
