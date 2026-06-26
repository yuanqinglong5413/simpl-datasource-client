use crate::connection_manager::ConnectionManager;
use crate::schema_cache::SchemaCache;
use simpl_driver_trait::{
    DriverErrorResponse, ExecuteResult, ExplainPlan, RowPage, SqlDriver, TablePageRequest,
};
use simpl_sql_utils::{split_statements, SqlDialectKind};
use uuid::Uuid;

/// 查询执行引擎：多语句拆分、Schema 缓存联动。
pub struct QueryEngine {
    connections: std::sync::Arc<ConnectionManager>,
    schema_cache: std::sync::Arc<SchemaCache>,
}

impl QueryEngine {
    pub fn new(
        connections: std::sync::Arc<ConnectionManager>,
        schema_cache: std::sync::Arc<SchemaCache>,
    ) -> Self {
        Self {
            connections,
            schema_cache,
        }
    }

    pub async fn execute_sql(
        &self,
        connection_id: &Uuid,
        sql: &str,
    ) -> Result<Vec<ExecuteResult>, DriverErrorResponse> {
        let dialect = self.dialect_for(connection_id).await?;
        let statements = split_statements(sql, dialect).map_err(|e| DriverErrorResponse {
            code: "parse_error".into(),
            user_message: format!("SQL 解析失败：{e}"),
            detail: Some(e),
        })?;

        let mut results = Vec::new();
        for stmt in statements {
            let result = self
                .connections
                .with_driver(connection_id, |driver| {
                    let stmt = stmt.clone();
                    Box::pin(async move { driver.execute(&stmt).await })
                })
                .await?;
            results.push(result);
        }
        Ok(results)
    }

    pub async fn fetch_table_page(
        &self,
        connection_id: &Uuid,
        req: TablePageRequest,
    ) -> Result<RowPage, DriverErrorResponse> {
        self.connections
            .with_driver(connection_id, |driver| {
                let req = req.clone();
                Box::pin(async move { driver.fetch_page(req).await })
            })
            .await
    }

    pub async fn explain_sql(
        &self,
        connection_id: &Uuid,
        sql: &str,
    ) -> Result<ExplainPlan, DriverErrorResponse> {
        self.connections
            .with_driver(connection_id, |driver| {
                let sql = sql.to_string();
                Box::pin(async move { driver.explain(&sql).await })
            })
            .await
    }

    pub async fn refresh_schema(
        &self,
        connection_id: &Uuid,
    ) -> Result<simpl_driver_trait::SchemaMeta, DriverErrorResponse> {
        let meta = self.connections.introspect(connection_id).await?;
        self.schema_cache.set(*connection_id, meta.clone()).await;
        Ok(meta)
    }

    pub async fn get_schema(
        &self,
        connection_id: &Uuid,
        force_refresh: bool,
    ) -> Result<simpl_driver_trait::SchemaMeta, DriverErrorResponse> {
        if !force_refresh {
            if let Some(cached) = self.schema_cache.get(connection_id).await {
                return Ok(cached);
            }
        }
        self.refresh_schema(connection_id).await
    }

    async fn dialect_for(
        &self,
        connection_id: &Uuid,
    ) -> Result<SqlDialectKind, DriverErrorResponse> {
        self.connections
            .with_driver(connection_id, |driver| {
                Box::pin(async move { Ok(SqlDialectKind::from(driver.dialect())) })
            })
            .await
    }
}
