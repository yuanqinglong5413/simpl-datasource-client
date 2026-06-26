use async_trait::async_trait;
use simpl_driver_mysql::MysqlDriver;
use simpl_driver_postgres::PostgresDriver;
use simpl_driver_sqlite::SqliteDriver;
use simpl_driver_trait::{
    ConnectionConfig, ConnectionSecrets, DatabaseKind, Dialect, DriverError, ExecuteResult,
    ExplainPlan, RowPage, SchemaMeta, SqlDriver, TablePageRequest,
};

/// 当前活跃的数据库驱动实例（枚举包装，便于 ConnectionManager 统一管理）。
pub enum ActiveDriver {
    Postgres(PostgresDriver),
    Mysql(MysqlDriver),
    Sqlite(SqliteDriver),
}

impl ActiveDriver {
    /// 根据连接配置创建并连接驱动（host/port 已由 SSH 解析层处理）。
    pub async fn connect(
        config: &ConnectionConfig,
        secrets: &ConnectionSecrets,
        host: &str,
        port: u16,
    ) -> Result<Self, DriverError> {
        match config.kind {
            DatabaseKind::Postgres => Ok(Self::Postgres(
                PostgresDriver::connect(config, secrets, host, port).await?,
            )),
            DatabaseKind::Mysql => Ok(Self::Mysql(
                MysqlDriver::connect(config, secrets, host, port).await?,
            )),
            DatabaseKind::Sqlite => Ok(Self::Sqlite(SqliteDriver::connect(config, secrets).await?)),
        }
    }
}

#[async_trait]
impl SqlDriver for ActiveDriver {
    fn dialect(&self) -> Dialect {
        match self {
            Self::Postgres(d) => d.dialect(),
            Self::Mysql(d) => d.dialect(),
            Self::Sqlite(d) => d.dialect(),
        }
    }

    async fn test_connection(&self) -> Result<(), DriverError> {
        match self {
            Self::Postgres(d) => d.test_connection().await,
            Self::Mysql(d) => d.test_connection().await,
            Self::Sqlite(d) => d.test_connection().await,
        }
    }

    async fn introspect(&self) -> Result<SchemaMeta, DriverError> {
        match self {
            Self::Postgres(d) => d.introspect().await,
            Self::Mysql(d) => d.introspect().await,
            Self::Sqlite(d) => d.introspect().await,
        }
    }

    async fn execute(&self, sql: &str) -> Result<ExecuteResult, DriverError> {
        match self {
            Self::Postgres(d) => d.execute(sql).await,
            Self::Mysql(d) => d.execute(sql).await,
            Self::Sqlite(d) => d.execute(sql).await,
        }
    }

    async fn fetch_page(&self, req: TablePageRequest) -> Result<RowPage, DriverError> {
        match self {
            Self::Postgres(d) => d.fetch_page(req).await,
            Self::Mysql(d) => d.fetch_page(req).await,
            Self::Sqlite(d) => d.fetch_page(req).await,
        }
    }

    async fn explain(&self, sql: &str) -> Result<ExplainPlan, DriverError> {
        match self {
            Self::Postgres(d) => d.explain(sql).await,
            Self::Mysql(d) => d.explain(sql).await,
            Self::Sqlite(d) => d.explain(sql).await,
        }
    }
}
