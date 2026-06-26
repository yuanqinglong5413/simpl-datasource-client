//! MySQL 驱动（sqlx）。

mod convert;
mod driver;

pub use driver::MysqlDriver;

use simpl_driver_trait::{ConnectionConfig, ConnectionSecrets, DriverError};

pub async fn connect(
    config: &ConnectionConfig,
    secrets: &ConnectionSecrets,
    host: &str,
    port: u16,
) -> Result<MysqlDriver, DriverError> {
    MysqlDriver::connect(config, secrets, host, port).await
}
