use crate::convert::{columns_from_row, row_to_cells};
use async_trait::async_trait;
use simpl_driver_trait::{
    ConnectionConfig, ConnectionSecrets, DatabaseKind, DriverError, Dialect, ExecuteResult,
    ExplainPlan, ExplainRow, RowPage, SchemaMeta, SqlDriver, TableMeta, TablePageRequest,
};
use sqlx::mysql::{MySqlPool, MySqlPoolOptions, MySqlRow};
use sqlx::{Column, Row};
use std::time::Instant;

pub struct MysqlDriver {
    pool: MySqlPool,
    database: String,
}

impl MysqlDriver {
    pub async fn connect(
        config: &ConnectionConfig,
        secrets: &ConnectionSecrets,
    ) -> Result<Self, DriverError> {
        if config.kind != DatabaseKind::Mysql {
            return Err(DriverError::InvalidConfig {
                user_message: "连接类型不是 MySQL".into(),
            });
        }
        let password = secrets.password.as_deref().unwrap_or("");
        let url = format!(
            "mysql://{}:{}@{}:{}/{}",
            urlencoding::encode(&config.username),
            urlencoding::encode(password),
            config.host,
            config.port,
            urlencoding::encode(&config.database)
        );
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| DriverError::ConnectionFailed {
                message: e.to_string(),
                user_message: format!("无法连接 MySQL：{e}"),
            })?;
        Ok(Self {
            pool,
            database: config.database.clone(),
        })
    }

    fn map_err(err: sqlx::Error) -> DriverError {
        DriverError::QueryFailed {
            message: err.to_string(),
            user_message: format!("查询执行失败：{err}"),
        }
    }
}

#[async_trait]
impl SqlDriver for MysqlDriver {
    fn dialect(&self) -> Dialect {
        Dialect::Mysql
    }

    async fn test_connection(&self) -> Result<(), DriverError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Self::map_err)
    }

    async fn introspect(&self) -> Result<SchemaMeta, DriverError> {
        let tables: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT table_name FROM information_schema.tables
            WHERE table_schema = DATABASE() AND table_type = 'BASE TABLE'
            ORDER BY table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_err)?;

        let mut table_metas = Vec::new();
        for table in tables {
            let columns = sqlx::query_as::<_, (String, String, String)>(
                r#"
                SELECT column_name, data_type, is_nullable
                FROM information_schema.columns
                WHERE table_schema = DATABASE() AND table_name = ?
                ORDER BY ordinal_position
                "#,
            )
            .bind(&table)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;

            let col_meta = columns
                .into_iter()
                .map(|(name, data_type, nullable)| simpl_driver_trait::ColumnMeta {
                    name: name.clone(),
                    data_type,
                    nullable: nullable == "YES",
                    is_primary_key: false,
                })
                .collect();

            table_metas.push(TableMeta {
                schema: self.database.clone(),
                name: table,
                columns: col_meta,
                primary_key: vec![],
                row_count: None,
            });
        }

        Ok(SchemaMeta {
            databases: vec![simpl_driver_trait::DatabaseMeta {
                name: self.database.clone(),
                schemas: vec![simpl_driver_trait::SchemaNode {
                    name: self.database.clone(),
                    tables: table_metas,
                    views: vec![],
                }],
            }],
        })
    }

    async fn execute(&self, sql: &str) -> Result<ExecuteResult, DriverError> {
        let start = Instant::now();
        let trimmed = sql.trim();
        let upper = trimmed.to_uppercase();
        let is_mutating = upper.starts_with("INSERT")
            || upper.starts_with("UPDATE")
            || upper.starts_with("DELETE")
            || upper.starts_with("CREATE")
            || upper.starts_with("ALTER")
            || upper.starts_with("DROP");

        if is_mutating {
            let result = sqlx::query(trimmed)
                .execute(&self.pool)
                .await
                .map_err(Self::map_err)?;
            return Ok(ExecuteResult {
                columns: vec![],
                rows: vec![],
                rows_affected: result.rows_affected(),
                execution_time_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("影响 {} 行", result.rows_affected())),
            });
        }

        let rows: Vec<MySqlRow> = sqlx::query(trimmed)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;
        let columns = rows.first().map(columns_from_row).unwrap_or_default();
        let data: Vec<Vec<simpl_driver_trait::CellValue>> =
            rows.iter().map(row_to_cells).collect();

        Ok(ExecuteResult {
            columns,
            rows: data.clone(),
            rows_affected: data.len() as u64,
            execution_time_ms: start.elapsed().as_millis() as u64,
            message: None,
        })
    }

    async fn fetch_page(&self, req: TablePageRequest) -> Result<RowPage, DriverError> {
        let order = req.order_by.as_deref().unwrap_or("1");
        let dir = if req.order_desc { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT * FROM `{}` ORDER BY `{}` {dir} LIMIT {} OFFSET {}",
            req.table, order, req.limit, req.offset
        );
        let rows: Vec<MySqlRow> = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;
        let columns = rows.first().map(columns_from_row).unwrap_or_default();
        Ok(RowPage {
            columns,
            rows: rows.iter().map(row_to_cells).collect(),
            total_rows: None,
            offset: req.offset,
            limit: req.limit,
        })
    }

    async fn explain(&self, sql: &str) -> Result<ExplainPlan, DriverError> {
        let rows: Vec<MySqlRow> = sqlx::query(&format!("EXPLAIN {sql}"))
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;
        let explain_rows = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                let mut fields = Vec::new();
                for (idx, col) in row.columns().iter().enumerate() {
                    let val: String = row.try_get(idx).unwrap_or_default();
                    fields.push((col.name().to_string(), simpl_driver_trait::CellValue::String(val)));
                }
                let _ = i;
                ExplainRow { fields }
            })
            .collect();
        Ok(ExplainPlan {
            rows: explain_rows,
            raw: None,
        })
    }
}
