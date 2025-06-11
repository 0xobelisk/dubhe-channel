//! Dubhe Channel Consensus
//!
//! 内部轻量 BFT / DAG 共识 (可选)

pub mod bft;
pub mod dag;
pub mod types;

pub use types::*;

use anyhow::Result;

/// 共识管理器
pub struct ConsensusManager {
    // TODO: 实现共识机制
}

impl ConsensusManager {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}
