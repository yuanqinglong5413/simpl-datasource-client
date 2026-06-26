use crate::driver_registry::ActiveDriver;
use crate::persistence::{ConnectionPersistence, PersistenceError};
use chrono::Utc;
use simpl_credential::CredentialStore;
use simpl_driver_trait::{
    ConnectionConfig, ConnectionSecrets, DriverError, DriverErrorResponse, SchemaMeta, SqlDriver,
};
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
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
}

/// 测试连接请求（未保存连接也可测试）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestConnectionRequest {
    pub config: ConnectionConfig,
    pub password: Option<String>,
}

/// 连接生命周期管理：持久化、凭证、活跃连接池。
pub struct ConnectionManager {
    data_dir: PathBuf,
    persistence: ConnectionPersistence,
    credentials: CredentialStore,
    active: Arc<RwLock<HashMap<Uuid, ActiveDriver>>>,
}

impl ConnectionManager {
    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        Self {
            persistence: ConnectionPersistence::new(&data_dir),
            credentials: CredentialStore::new(&data_dir),
            data_dir,
            active: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn uses_keyring(&self) -> bool {
        self.credentials.uses_keyring()
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
        config.updated_at = Utc::now();

        if let Some(password) = req.password.filter(|p| !p.is_empty()) {
            self.credentials
                .store_secret(&config.id, &password)
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
        Ok(())
    }

    pub async fn test_connection(
        &self,
        req: TestConnectionRequest,
    ) -> Result<(), DriverErrorResponse> {
        let secrets = ConnectionSecrets {
            password: req.password.or_else(|| {
                self.credentials
                    .get_secret(&req.config.id)
                    .ok()
                    .flatten()
            }),
            ssh_password: None,
        };
        let driver = ActiveDriver::connect(&req.config, &secrets)
            .await
            .map_err(DriverErrorResponse::from)?;
        driver
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
        let secrets = ConnectionSecrets {
            password: self
                .credentials
                .get_secret(id)
                .ok()
                .flatten(),
            ssh_password: None,
        };
        let driver = ActiveDriver::connect(&config, &secrets)
            .await
            .map_err(DriverErrorResponse::from)?;
        driver
            .test_connection()
            .await
            .map_err(DriverErrorResponse::from)?;
        self.active.write().await.insert(*id, driver);
        Ok(config)
    }

    pub async fn disconnect(&self, id: &Uuid) {
        self.active.write().await.remove(id);
    }

    pub async fn introspect(&self, id: &Uuid) -> Result<SchemaMeta, DriverErrorResponse> {
        self.with_driver(id, |driver| Box::pin(async move { driver.introspect().await }))
            .await
    }

    /// 在已连接驱动上执行异步操作。
    pub async fn with_driver<F, T>(&self, id: &Uuid, f: F) -> Result<T, DriverErrorResponse>
    where
        F: for<'a> FnOnce(
            &'a ActiveDriver,
        ) -> Pin<Box<dyn Future<Output = Result<T, DriverError>> + Send + 'a>>,
    {
        let active = self.active.read().await;
        let driver = active.get(id).ok_or(DriverErrorResponse {
            code: "not_connected".into(),
            user_message: "请先连接数据库".into(),
            detail: None,
        })?;
        f(driver).await.map_err(DriverErrorResponse::from)
    }
}
