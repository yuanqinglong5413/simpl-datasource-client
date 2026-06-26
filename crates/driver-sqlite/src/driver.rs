use crate::convert::{columns_from_row, row_to_cells};
use async_trait::async_trait;
use simpl_driver_trait::{
    ConnectionConfig, ConnectionSecrets, DatabaseKind, DriverError, Dialect, ExecuteResult,
    ExplainPlan, ExplainRow, RowPage, SchemaMeta, SqlDriver, TableMeta, TablePageRequest,
};
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions, SqliteRow};
use sqlx::Row;
use std::time::Instant;

pub struct SqliteDriver {
    pool: SqlitePool,
    path: String,
}

impl SqliteDriver {
    pub async fn connect(
        config: &ConnectionConfig,
        secrets: &ConnectionSecrets,
    ) -> Result<Self, DriverError> {
        let _ = secrets;
        if config.kind != DatabaseKind::Sqlite {
            return Err(DriverError::InvalidConfig {
                user_message: "连接类型不是 SQLite".into(),
            });
        }
        let path = if config.database.is_empty() {
            config.host.clone()
        } else {
            config.database.clone()
        };
        let url = format!("sqlite://{}", path);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| DriverError::ConnectionFailed {
                message: e.to_string(),
                user_message: format!("无法打开 SQLite 文件：{e}"),
            })?;
        Ok(Self { pool, path })
    }

    fn map_err(err: sqlx::Error) -> DriverError {
        DriverError::QueryFailed {
            message: err.to_string(),
            user_message: format!("查询执行失败：{err}"),
        }
    }
}

#[async_trait]
impl SqlDriver for SqliteDriver {
    fn dialect(&self) -> Dialect {
        Dialect::Sqlite
    }

    async fn test_connection(&self) -> Result<(), DriverError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Self::map_err)
    }

    async fn introspect(&self) -> Result<SchemaMeta, DriverError> {
        let tables: Vec<String> =
            sqlx::query_scalar("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
                .fetch_all(&self.pool)
                .await
                .map_err(Self::map_err)?;

        let mut table_metas = Vec::new();
        for table in tables {
            let pragma = format!("PRAGMA table_info(\"{table}\")");
            let columns = sqlx::query_as::<_, (i32, String, String, i32, Option<String>, i32)>(
                &pragma,
            )
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;

            let pk: Vec<String> = columns
                .iter()
                .filter(|c| c.5 == 1)
                .map(|c| c.1.clone())
                .collect();

            let col_meta = columns
                .into_iter()
                .map(|(_, name, data_type, notnull, _, pk_flag)| {
                    simpl_driver_trait::ColumnMeta {
                        name: name.clone(),
                        data_type,
                        nullable: notnull == 0,
                        is_primary_key: pk_flag == 1,
                    }
                })
                .collect();

            table_metas.push(TableMeta {
                schema: "main".into(),
                name: table,
                columns: col_meta,
                primary_key: pk,
                row_count: None,
            });
        }

        Ok(SchemaMeta {
            databases: vec![simpl_driver_trait::DatabaseMeta {
                name: self.path.clone(),
                schemas: vec![simpl_driver_trait::SchemaNode {
                    name: "main".into(),
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

        let rows: Vec<SqliteRow> = sqlx::query(trimmed)
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
        let sql = format!(
            "SELECT * FROM \"{}\" LIMIT {} OFFSET {}",
            req.table, req.limit, req.offset
        );
        let rows: Vec<SqliteRow> = sqlx::query(&sql)
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
        let rows: Vec<SqliteRow> = sqlx::query(&format!("EXPLAIN QUERY PLAN {sql}"))
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_err)?;
        let explain_rows = rows
            .iter()
            .map(|row| {
                let detail: String = row.try_get(3).unwrap_or_default();
                ExplainRow {
                    fields: vec![(
                        "detail".into(),
                        simpl_driver_trait::CellValue::String(detail),
                    )],
                }
            })
            .collect();
        Ok(ExplainPlan {
            rows: explain_rows,
            raw: None,
        })
    }
}
