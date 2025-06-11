//! Loader 错误类型

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoaderError {
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Plugin error: {0}")]
    PluginError(String),

    #[error("Unsupported contract type: {0:?}")]
    UnsupportedContractType(dubhe_adapter::ContractType),

    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rocksdb::Error),
}
