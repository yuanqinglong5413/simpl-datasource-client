use crate::connect_resolve::ResolvedConnection;
use crate::dml_generator::{generate_update, CellChangeRequest, DmlPreview};
use crate::driver_registry::ActiveDriver;
use crate::export::{export_row_page, ExportFormat, ExportResult};
use crate::import::{build_import_sql, preview_import, ImportFileFormat, ImportPreview, ImportRequest, ImportResult};
use crate::persistence::{ConnectionPersistence, PersistenceError};
use crate::transaction_manager::{TransactionManager, TransactionStatus};
use simpl_credential::CredentialStore;
use simpl_driver_trait::{
    ConnectionConfig, ConnectionSecrets, DriverError, DriverErrorResponse, RowPage, SchemaMeta,
    SqlDriver, TablePageRequest,
};
use simpl_ssh::SshTunnel;
use std::collections::HashMap;
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// 对外暴露的连接摘要（列表页使用，不含敏感字段）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionRecord {
    pub config: ConnectionConfig,
    pub connected: bool,
}

/// 保存/更新连接请求。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SaveConnectionRequest {
    pub config: ConnectionConfig,
    pub password: Option<String>,
    pub ssh_password: Option<String>,
}

/// 测试连接请求（未保存连接也可测试）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestConnectionRequest {
    pub config: ConnectionConfig,
    pub password: Option<String>,
    pub ssh_password: Option<String>,
}

/// 活跃连接（驱动 + SSH 隧道句柄）。
struct ActiveConnection {
    driver: ActiveDriver,
    /// 保持隧道存活；断连时 drop。
    _tunnel: Option<SshTunnel>,
}

/// 连接生命周期管理：持久化、凭证、SSH 隧道、Safe Mode 事务。
pub struct ConnectionManager {
    data_dir: std::path::PathBuf,
    persistence: ConnectionPersistence,
    credentials: CredentialStore,
    active: Arc<RwLock<HashMap<Uuid, ActiveConnection>>>,
    transactions: Arc<RwLock<TransactionManager>>,
}

impl ConnectionManager {
    pub fn new(data_dir: impl AsRef<std::path::Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        Self {
            persistence: ConnectionPersistence::new(&data_dir),
            credentials: CredentialStore::new(&data_dir),
            data_dir,
            active: Arc::new(RwLock::new(HashMap::new())),
            transactions: Arc::new(RwLock::new(TransactionManager::new())),
        }
    }

    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    pub fn uses_keyring(&self) -> bool {
        self.credentials.uses_keyring()
    }

    pub fn transactions(&self) -> Arc<RwLock<TransactionManager>> {
        self.transactions.clone()
    }

    pub async fn list_connections(&self) -> Result<Vec<ConnectionRecord>, PersistenceError> {
        let configs = self.persistence.load_all()?;
        let active = self.active.read().await;
        Ok(configs
            .into_iter()
            .map(|config| ConnectionRecord {
                connected: active.contains_key(&config.id),
                config,
            })
            .collect())
    }

    pub fn save_connection(
        &self,
        req: SaveConnectionRequest,
    ) -> Result<ConnectionConfig, PersistenceError> {
        let mut all = self.persistence.load_all()?;
        let mut config = req.config;
        config.updated_at = chrono::Utc::now();

        if let Some(password) = req.password.filter(|p| !p.is_empty()) {
            self.credentials
                .store_secret(&config.id, &password)
                .map_err(|e| PersistenceError::Write(e.to_string()))?;
        }
        if let Some(ssh_password) = req.ssh_password.filter(|p| !p.is_empty()) {
            self.credentials
                .store_ssh_secret(&config.id, &ssh_password)
                .map_err(|e| PersistenceError::Write(e.to_string()))?;
        }

        if let Some(pos) = all.iter().position(|c| c.id == config.id) {
            all[pos] = config.clone();
        } else {
            all.push(config.clone());
        }
        self.persistence.save_all(&all)?;
        Ok(config)
    }

    pub fn delete_connection(&self, id: &Uuid) -> Result<(), PersistenceError> {
        let mut all = self.persistence.load_all()?;
        let len_before = all.len();
        all.retain(|c| &c.id != id);
        if all.len() == len_before {
            return Err(PersistenceError::NotFound);
        }
        self.persistence.save_all(&all)?;
        let _ = self.credentials.delete_secret(id);
        let _ = self.credentials.delete_ssh_secret(id);
        Ok(())
    }

    fn build_secrets(
        &self,
        config: &ConnectionConfig,
        password: Option<String>,
        ssh_password: Option<String>,
    ) -> ConnectionSecrets {
        ConnectionSecrets {
            password: password.or_else(|| {
                self.credentials
                    .get_secret(&config.id)
                    .ok()
                    .flatten()
            }),
            ssh_password: ssh_password.or_else(|| {
                self.credentials
                    .get_ssh_secret(&config.id)
                    .ok()
                    .flatten()
            }),
        }
    }

    async fn open_driver(
        &self,
        config: &ConnectionConfig,
        secrets: &ConnectionSecrets,
    ) -> Result<ActiveConnection, DriverErrorResponse> {
        let resolved = ResolvedConnection::resolve(config, secrets).map_err(|e| DriverErrorResponse {
            code: "ssh_failed".into(),
            user_message: format!("SSH 隧道建立失败：{e}"),
            detail: Some(e.to_string()),
        })?;
        let driver = ActiveDriver::connect(config, secrets, &resolved.host, resolved.port)
            .await
            .map_err(DriverErrorResponse::from)?;
        Ok(ActiveConnection {
            driver,
            _tunnel: resolved.tunnel,
        })
    }

    pub async fn test_connection(
        &self,
        req: TestConnectionRequest,
    ) -> Result<(), DriverErrorResponse> {
        let secrets = self.build_secrets(&req.config, req.password, req.ssh_password);
        let conn = self.open_driver(&req.config, &secrets).await?;
        conn.driver
            .test_connection()
            .await
            .map_err(DriverErrorResponse::from)
    }

    pub async fn connect(&self, id: &Uuid) -> Result<ConnectionConfig, DriverErrorResponse> {
        let config = self
            .persistence
            .get(id)
            .map_err(|_| DriverErrorResponse {
                code: "not_found".into(),
                user_message: "连接不存在".into(),
                detail: None,
            })?;
        let secrets = self.build_secrets(&config, None, None);
        let conn = self.open_driver(&config, &secrets).await?;
        conn.driver
            .test_connection()
            .await
            .map_err(DriverErrorResponse::from)?;
        self.active.write().await.insert(*id, conn);
        Ok(config)
    }

    pub async fn disconnect(&self, id: &Uuid) {
        self.active.write().await.remove(id);
        self.transactions.write().await.clear(id);
    }

    pub async fn introspect(&self, id: &Uuid) -> Result<SchemaMeta, DriverErrorResponse> {
        self.with_driver(id, |driver| Box::pin(async move { driver.introspect().await }))
            .await
    }

    pub async fn fetch_table_page(
        &self,
        id: &Uuid,
        req: TablePageRequest,
    ) -> Result<RowPage, DriverErrorResponse> {
        self.with_driver(id, |driver| {
            let req = req.clone();
            Box::pin(async move { driver.fetch_page(req).await })
        })
        .await
    }

    pub async fn preview_cell_change(
        &self,
        id: &Uuid,
        change: CellChangeRequest,
    ) -> Result<DmlPreview, DriverErrorResponse> {
        let dialect = self
            .with_driver(id, |driver| {
                Box::pin(async move { Ok(driver.dialect()) })
            })
            .await?;
        generate_update(dialect, &change).map_err(|e| DriverErrorResponse {
            code: "invalid_change".into(),
            user_message: e,
            detail: None,
        })
    }

    pub async fn queue_cell_change(
        &self,
        id: &Uuid,
        change: CellChangeRequest,
    ) -> Result<DmlPreview, DriverErrorResponse> {
        let preview = self.preview_cell_change(id, change).await?;
        self.transactions
            .write()
            .await
            .queue_sql(*id, preview.sql.clone());
        Ok(preview)
    }

    pub async fn commit_transaction(&self, id: &Uuid) -> Result<u64, DriverErrorResponse> {
        let sqls = self.transactions.read().await.pending_sql(id);
        if sqls.is_empty() {
            return Ok(0);
        }
        let affected = self
            .with_driver(id, |driver| {
                let sqls = sqls.clone();
                Box::pin(async move {
                    driver.execute("BEGIN").await?;
                    let mut total = 0u64;
                    for sql in &sqls {
                        total += driver.execute(sql).await?.rows_affected;
                    }
                    driver.execute("COMMIT").await?;
                    Ok(total)
                })
            })
            .await?;
        self.transactions.write().await.clear(id);
        Ok(affected)
    }

    pub async fn rollback_transaction(&self, id: &Uuid) -> Result<(), DriverErrorResponse> {
        let sqls = self.transactions.read().await.pending_sql(id);
        if !sqls.is_empty() {
            let _ = self
                .with_driver(id, |driver| {
                    Box::pin(async move {
                        let _ = driver.execute("ROLLBACK").await;
                        Ok(())
                    })
                })
                .await;
        }
        self.transactions.write().await.clear(id);
        Ok(())
    }

    pub async fn transaction_status(&self, id: &Uuid) -> TransactionStatus {
        self.transactions.read().await.status(id)
    }

    pub fn export_page(
        &self,
        page: &RowPage,
        format: ExportFormat,
        path: &Path,
    ) -> Result<ExportResult, String> {
        export_row_page(page, format, path)
    }

    pub async fn import_preview(
        &self,
        id: &Uuid,
        path: &Path,
        format: ImportFileFormat,
        table: &str,
        schema: Option<String>,
    ) -> Result<ImportPreview, DriverErrorResponse> {
        let meta = self.introspect(id).await?;
        let columns = meta
            .databases
            .iter()
            .flat_map(|db| db.schemas.iter())
            .flat_map(|sch| sch.tables.iter())
            .find(|t| {
                t.name == table
                    && schema
                        .as_ref()
                        .map(|s| &t.schema == s)
                        .unwrap_or(true)
            })
            .map(|t| t.columns.clone())
            .unwrap_or_default();
        preview_import(path, format, &columns).map_err(|e| DriverErrorResponse {
            code: "import_preview_failed".into(),
            user_message: e.clone(),
            detail: Some(e),
        })
    }

    pub async fn execute_import(
        &self,
        id: &Uuid,
        request: ImportRequest,
    ) -> Result<ImportResult, DriverErrorResponse> {
        let built = build_import_sql(&request).map_err(|e| DriverErrorResponse {
            code: "import_build_failed".into(),
            user_message: e.clone(),
            detail: Some(e),
        })?;
        for sql in &built.insert_statements {
            self.with_driver(id, |driver| {
                let sql = sql.clone();
                Box::pin(async move { driver.execute(&sql).await })
            })
            .await?;
        }
        Ok(built)
    }

    /// 在已连接驱动上执行异步操作。
    pub async fn with_driver<F, T>(&self, id: &Uuid, f: F) -> Result<T, DriverErrorResponse>
    where
        F: for<'a> FnOnce(
            &'a ActiveDriver,
        ) -> Pin<Box<dyn Future<Output = Result<T, DriverError>> + Send + 'a>>,
    {
        let active = self.active.read().await;
        let conn = active.get(id).ok_or(DriverErrorResponse {
            code: "not_connected".into(),
            user_message: "请先连接数据库".into(),
            detail: None,
        })?;
        f(&conn.driver).await.map_err(DriverErrorResponse::from)
    }
}
