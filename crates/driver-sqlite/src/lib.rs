//! SQLite 驱动（sqlx）。

mod convert;
mod driver;

pub use driver::SqliteDriver;

use simpl_driver_trait::{ConnectionConfig, ConnectionSecrets, DriverError};

pub async fn connect(
    config: &ConnectionConfig,
    secrets: &ConnectionSecrets,
) -> Result<SqliteDriver, DriverError> {
    SqliteDriver::connect(config, secrets).await
}
