use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 驱动层统一错误，携带用户可读消息供前端 i18n 映射。
#[derive(Debug, Error)]
pub enum DriverError {
    #[error("connection failed: {message}")]
    ConnectionFailed {
        message: String,
        user_message: String,
    },

    #[error("query failed: {message}")]
    QueryFailed {
        message: String,
        user_message: String,
    },

    #[error("unsupported operation: {user_message}")]
    Unsupported { user_message: String },

    #[error("invalid configuration: {user_message}")]
    InvalidConfig { user_message: String },

    #[error("internal error: {message}")]
    Internal {
        message: String,
        user_message: String,
    },
}

impl DriverError {
    /// 返回面向用户的错误描述（默认中文）。
    pub fn user_message(&self) -> &str {
        match self {
            Self::ConnectionFailed { user_message, .. }
            | Self::QueryFailed { user_message, .. }
            | Self::Unsupported { user_message }
            | Self::InvalidConfig { user_message }
            | Self::Internal { user_message, .. } => user_message,
        }
    }
}

/// 可序列化的错误响应，用于 IPC 传输。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverErrorResponse {
    pub code: String,
    pub user_message: String,
    pub detail: Option<String>,
}

impl From<&DriverError> for DriverErrorResponse {
    fn from(err: &DriverError) -> Self {
        let (code, detail) = match err {
            DriverError::ConnectionFailed { message, .. } => ("connection_failed", Some(message.clone())),
            DriverError::QueryFailed { message, .. } => ("query_failed", Some(message.clone())),
            DriverError::Unsupported { .. } => ("unsupported", None),
            DriverError::InvalidConfig { .. } => ("invalid_config", None),
            DriverError::Internal { message, .. } => ("internal", Some(message.clone())),
        };
        Self {
            code: code.to_string(),
            user_message: err.user_message().to_string(),
            detail,
        }
    }
}

impl From<DriverError> for DriverErrorResponse {
    fn from(err: DriverError) -> Self {
        DriverErrorResponse::from(&err)
    }
}
