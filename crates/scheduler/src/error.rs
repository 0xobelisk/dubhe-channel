//! Scheduler 错误类型

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Conflict detection failed: {0}")]
    ConflictDetectionFailed(String),

    #[error("Strategy error: {0}")]
    StrategyError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}
