//! 解析连接参数：可选 SSH 隧道后将数据库 host/port 映射到本地。

use simpl_driver_trait::{ConnectionConfig, ConnectionSecrets, DatabaseKind};
use simpl_ssh::{SshError, SshTunnel};

/// 解析后的连接目标（含可选 SSH 隧道句柄）。
pub struct ResolvedConnection {
    /// 供驱动使用的 host（可能为 127.0.0.1）
    pub host: String,
    pub port: u16,
    pub tunnel: Option<SshTunnel>,
}

impl ResolvedConnection {
    /// 根据配置建立 SSH 隧道（若启用）并返回实际连接地址。
    pub fn resolve(
        config: &ConnectionConfig,
        secrets: &ConnectionSecrets,
    ) -> Result<Self, SshError> {
        if config.kind == DatabaseKind::Sqlite || !config.ssh.enabled {
            return Ok(Self {
                host: config.host.clone(),
                port: config.port,
                tunnel: None,
            });
        }

        let tunnel = SshTunnel::connect(
            &config.ssh,
            secrets,
            &config.host,
            config.port,
        )?;
        Ok(Self {
            host: "127.0.0.1".to_string(),
            port: tunnel.local_port(),
            tunnel: Some(tunnel),
        })
    }
}
