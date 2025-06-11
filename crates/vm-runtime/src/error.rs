//! VM Runtime 错误类型

use thiserror::Error;

#[derive(Error, Debug)]
pub enum VmError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Code loading failed: {0}")]
    CodeLoadingFailed(String),

    #[error("VM initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Snapshot operation failed: {0}")]
    SnapshotFailed(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}
