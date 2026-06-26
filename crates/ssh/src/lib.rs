//! SSH 本地端口转发：将 127.0.0.1:local_port 转发到 remote_host:remote_port。

use simpl_driver_trait::{ConnectionSecrets, SshConfig};
use ssh2::Session;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SshError {
    #[error("ssh connect failed: {0}")]
    Connect(String),
    #[error("ssh auth failed: {0}")]
    Auth(String),
    #[error("ssh tunnel failed: {0}")]
    Tunnel(String),
}

/// 活跃 SSH 隧道；Drop 时关闭本地监听与转发线程。
pub struct SshTunnel {
    local_port: u16,
    stop: Arc<std::sync::atomic::AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl SshTunnel {
    /// 建立 SSH 隧道并绑定本地随机端口。
    pub fn connect(
        ssh: &SshConfig,
        secrets: &ConnectionSecrets,
        remote_host: &str,
        remote_port: u16,
    ) -> Result<Self, SshError> {
        if !ssh.enabled {
            return Err(SshError::Tunnel("SSH 未启用".into()));
        }
        if ssh.host.is_empty() || ssh.username.is_empty() {
            return Err(SshError::Connect("SSH 主机或用户名为空".into()));
        }

        let tcp = TcpStream::connect(format!("{}:{}", ssh.host, ssh.port))
            .map_err(|e| SshError::Connect(format!("无法连接 SSH 服务器：{e}")))?;
        tcp.set_read_timeout(Some(Duration::from_secs(30)))
            .ok();
        tcp.set_write_timeout(Some(Duration::from_secs(30)))
            .ok();

        let mut sess = Session::new().map_err(|e| SshError::Connect(e.to_string()))?;
        sess.set_tcp_stream(tcp);
        sess.handshake()
            .map_err(|e| SshError::Connect(format!("SSH 握手失败：{e}")))?;

        // 认证：密钥优先，其次密码
        if let Some(ref key_path) = ssh.private_key_path {
            if !key_path.is_empty() {
                sess.userauth_pubkey_file(
                    &ssh.username,
                    None,
                    std::path::Path::new(key_path),
                    secrets.ssh_password.as_deref(),
                )
                .map_err(|e| SshError::Auth(format!("SSH 密钥认证失败：{e}")))?;
            }
        }

        if !sess.authenticated() {
            let pwd = secrets.ssh_password.as_deref().unwrap_or("");
            if pwd.is_empty() {
                return Err(SshError::Auth("需要提供 SSH 密码或私钥".into()));
            }
            sess.userauth_password(&ssh.username, pwd)
                .map_err(|e| SshError::Auth(format!("SSH 密码认证失败：{e}")))?;
        }

        if !sess.authenticated() {
            return Err(SshError::Auth("SSH 认证未成功".into()));
        }

        let listener = TcpListener::bind("127.0.0.1:0")
            .map_err(|e| SshError::Tunnel(format!("无法绑定本地端口：{e}")))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| SshError::Tunnel(e.to_string()))?;
        let local_port = listener
            .local_addr()
            .map_err(|e| SshError::Tunnel(e.to_string()))?
            .port();

        let session = Arc::new(Mutex::new(sess));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_flag = stop.clone();
        let target_host = remote_host.to_string();
        let target_port = remote_port;

        let thread = thread::spawn(move || {
            while !stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut local, _)) => {
                        let session = session.clone();
                        let host = target_host.clone();
                        thread::spawn(move || {
                            if let Err(e) = forward_connection(session, &host, target_port, &mut local)
                            {
                                tracing::debug!("ssh forward ended: {e}");
                            }
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(e) => {
                        tracing::warn!("ssh listener error: {e}");
                        break;
                    }
                }
            }
        });

        Ok(Self {
            local_port,
            stop,
            thread: Some(thread),
        })
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }
}

impl Drop for SshTunnel {
    fn drop(&mut self) {
        self.stop
            .store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

fn forward_connection(
    session: Arc<Mutex<Session>>,
    target_host: &str,
    target_port: u16,
    local: &mut TcpStream,
) -> Result<(), String> {
    let mut channel = {
        let sess = session.lock().map_err(|e| e.to_string())?;
        sess.channel_direct_tcpip(target_host, target_port, None)
            .map_err(|e| e.to_string())?
    };

    let mut local_read = local.try_clone().map_err(|e| e.to_string())?;

    let mut buf = [0u8; 8192];
    loop {
        if local_read.read(&mut buf).is_ok_and(|n| {
            if n == 0 {
                return true;
            }
            channel.write_all(&buf[..n]).is_err()
        }) {
            break;
        }
        if channel.read(&mut buf).is_ok_and(|n| {
            if n == 0 {
                return true;
            }
            local.write_all(&buf[..n]).is_err()
        }) {
            break;
        }
    }
    let _ = channel.send_eof();
    let _ = channel.wait_eof();
    let _ = channel.close();
    let _ = channel.wait_close();
    Ok(())
}
