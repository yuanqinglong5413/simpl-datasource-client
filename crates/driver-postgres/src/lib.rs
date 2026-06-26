//! PostgreSQL 驱动实现（sqlx）。

mod convert;
mod driver;

pub use driver::PostgresDriver;

use simpl_driver_trait::{ConnectionConfig, ConnectionSecrets, DriverError};

/// 根据连接配置创建 PostgreSQL 驱动实例。
pub async fn connect(
    config: &ConnectionConfig,
    secrets: &ConnectionSecrets,
) -> Result<PostgresDriver, DriverError> {
    PostgresDriver::connect(config, secrets).await
}
