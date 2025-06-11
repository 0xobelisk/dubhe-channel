//! Dubhe Channel State
//!
//! 存储层 (RocksDB) + 索引

pub mod indexer;
pub mod storage;
pub mod types;

pub use indexer::*;
pub use storage::*;
pub use types::*;

use anyhow::Result;

/// 状态管理器
pub struct StateManager {
    // TODO: 实现状态管理
}

impl StateManager {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
