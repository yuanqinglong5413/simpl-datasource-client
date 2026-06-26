use crate::convert::{columns_from_row, row_to_cells};
use async_trait::async_trait;
use simpl_driver_trait::{
    ConnectionConfig, ConnectionSecrets, DatabaseKind, DriverError, Dialect, ExecuteResult,
    ExplainPlan, ExplainRow, RowPage, SchemaMeta, SqlDriver, TableMeta, TablePageRequest,
};
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use std::time::Instant;

/// PostgreSQL 连接驱动。
pub struct PostgresDriver {
    pool: PgPool,
    database: String,
}

impl PostgresDriver {
    pub async fn connect(
        config: &ConnectionConfig,
        secrets: &ConnectionSecrets,
        host: &str,
        port: u16,
    ) -> Result<Self, DriverError> {
        if config.kind != DatabaseKind::Postgres {
            return Err(DriverError::InvalidConfig {
                user_message: "连接类型不是 PostgreSQL".into(),
            });
        }
        let password = secrets.password.as_deref().unwrap_or("");
        let url = format!(
            "postgres://{}:{}@{}:{}/{}",
            urlencoding_encode(&config.username),
            urlencoding_encode(password),
            host,
            port,
            urlencoding_encode(&config.database)
        );
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await
            .map_err(|e| DriverError::ConnectionFailed {
                message: e.to_string(),
                user_message: format!("无法连接 PostgreSQL：{}", e),
            })?;
        Ok(Self {
            pool,
            database: config.database.clone(),
        })
    }

    fn map_query_error(err: sqlx::Error) -> DriverError {
        DriverError::QueryFailed {
            message: err.to_string(),
            user_message: format!("查询执行失败：{err}"),
        }
    }
}

fn urlencoding_encode(s: &str) -> String {
    urlencoding::encode(s).into_owned()
}

#[async_trait]
impl SqlDriver for PostgresDriver {
    fn dialect(&self) -> Dialect {
        Dialect::Postgres
    }

    async fn test_connection(&self) -> Result<(), DriverError> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(Self::map_query_error)
    }

    async fn introspect(&self) -> Result<SchemaMeta, DriverError> {
        let tables: Vec<(String, String)> = sqlx::query_as(
            r#"
            SELECT table_schema, table_name
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
              AND table_type = 'BASE TABLE'
            ORDER BY table_schema, table_name
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Self::map_query_error)?;

        let mut schema_map: std::collections::BTreeMap<String, Vec<TableMeta>> =
            std::collections::BTreeMap::new();

        for (schema, table) in tables {
            let columns = sqlx::query_as::<_, (String, String, String)>(
                r#"
                SELECT column_name, data_type, is_nullable
                FROM information_schema.columns
                WHERE table_schema = $1 AND table_name = $2
                ORDER BY ordinal_position
                "#,
            )
            .bind(&schema)
            .bind(&table)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_query_error)?;

            let pk_cols: Vec<String> = sqlx::query_scalar(
                r#"
                SELECT kcu.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage kcu
                  ON tc.constraint_name = kcu.constraint_name
                 AND tc.table_schema = kcu.table_schema
                WHERE tc.constraint_type = 'PRIMARY KEY'
                  AND tc.table_schema = $1 AND tc.table_name = $2
                ORDER BY kcu.ordinal_position
                "#,
            )
            .bind(&schema)
            .bind(&table)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_query_error)?;

            let col_meta: Vec<simpl_driver_trait::ColumnMeta> = columns
                .into_iter()
                .map(|(name, data_type, nullable)| simpl_driver_trait::ColumnMeta {
                    name: name.clone(),
                    data_type,
                    nullable: nullable == "YES",
                    is_primary_key: pk_cols.contains(&name),
                })
                .collect();

            schema_map.entry(schema.clone()).or_default().push(TableMeta {
                schema: schema.clone(),
                name: table,
                columns: col_meta,
                primary_key: pk_cols,
                row_count: None,
            });
        }

        let schemas = schema_map
            .into_iter()
            .map(|(name, tables)| simpl_driver_trait::SchemaNode {
                name,
                tables,
                views: vec![],
            })
            .collect();

        Ok(SchemaMeta {
            databases: vec![simpl_driver_trait::DatabaseMeta {
                name: self.database.clone(),
                schemas,
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
                .map_err(Self::map_query_error)?;
            return Ok(ExecuteResult {
                columns: vec![],
                rows: vec![],
                rows_affected: result.rows_affected(),
                execution_time_ms: start.elapsed().as_millis() as u64,
                message: Some(format!("影响 {} 行", result.rows_affected())),
            });
        }

        let rows: Vec<PgRow> = sqlx::query(trimmed)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_query_error)?;

        let columns = rows
            .first()
            .map(columns_from_row)
            .unwrap_or_default();
        let data: Vec<Vec<simpl_driver_trait::CellValue>> =
            rows.iter().map(row_to_cells).collect();
        let count = data.len() as u64;

        Ok(ExecuteResult {
            columns,
            rows: data,
            rows_affected: count,
            execution_time_ms: start.elapsed().as_millis() as u64,
            message: None,
        })
    }

    async fn fetch_page(&self, req: TablePageRequest) -> Result<RowPage, DriverError> {
        let schema = req.schema.as_deref().unwrap_or("public");
        let order = req
            .order_by
            .as_deref()
            .map(|c| format!("\"{c}\""))
            .unwrap_or_else(|| "(SELECT NULL)".into());
        let dir = if req.order_desc { "DESC" } else { "ASC" };
        let sql = format!(
            "SELECT * FROM \"{schema}\".\"{}\" ORDER BY {order} {dir} LIMIT {} OFFSET {}",
            req.table, req.limit, req.offset
        );
        let rows: Vec<PgRow> = sqlx::query(&sql)
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_query_error)?;
        let columns = rows
            .first()
            .map(columns_from_row)
            .unwrap_or_default();
        Ok(RowPage {
            columns,
            rows: rows.iter().map(row_to_cells).collect(),
            total_rows: None,
            offset: req.offset,
            limit: req.limit,
        })
    }

    async fn explain(&self, sql: &str) -> Result<ExplainPlan, DriverError> {
        let rows: Vec<PgRow> = sqlx::query(&format!("EXPLAIN {sql}"))
            .fetch_all(&self.pool)
            .await
            .map_err(Self::map_query_error)?;
        let explain_rows: Vec<ExplainRow> = rows
            .iter()
            .map(|row| {
                let plan: String = row.try_get(0).unwrap_or_default();
                ExplainRow {
                    fields: vec![("QUERY PLAN".into(), simpl_driver_trait::CellValue::String(plan))],
                }
            })
            .collect();
        Ok(ExplainPlan {
            rows: explain_rows,
            raw: None,
        })
    }
}
