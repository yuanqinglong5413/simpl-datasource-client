//! 简源（SimplSource）数据库驱动抽象层。
//!
//! 定义跨 GUI / CLI / MCP 共享的类型与 [`SqlDriver`] trait 契约。

mod error;
mod types;

pub use error::{DriverError, DriverErrorResponse};
pub use types::*;

use async_trait::async_trait;

/// 异步 SQL 驱动统一接口；各数据库 crate 实现此 trait。
#[async_trait]
pub trait SqlDriver: Send + Sync {
    /// 返回当前连接方言。
    fn dialect(&self) -> Dialect;

    /// 测试连接是否可用。
    async fn test_connection(&self) -> Result<(), DriverError>;

    /// 拉取 Schema 元数据（库/表/列）。
    async fn introspect(&self) -> Result<SchemaMeta, DriverError>;

    /// 执行任意 SQL（查询或变更）。
    async fn execute(&self, sql: &str) -> Result<ExecuteResult, DriverError>;

    /// 分页读取表数据。
    async fn fetch_page(&self, req: TablePageRequest) -> Result<RowPage, DriverError>;

    /// 获取执行计划（方言相关，不支持时返回 Unsupported）。
    async fn explain(&self, sql: &str) -> Result<ExplainPlan, DriverError>;
}
